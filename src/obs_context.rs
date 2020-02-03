use std::collections::BTreeMap;
use std::fmt;
use std::path::*;

use fitsio::*;

use crate::fits_read::*;
use crate::gpubox::*;
use crate::*;

/// `mwalib` observation context.
///
/// The name is not following the rust convention of camel case, to make it look
/// more like a C library.
#[allow(non_camel_case_types)]
pub struct mwalibObsContext {
    pub obsid: u32,
    pub start_time_milliseconds: u64,
    // `end_time_milliseconds` will reflect the start time of the *last* HDU it
    // is derived from (i.e. `end_time_milliseconds` + integration time is the
    // actual end time of the observation).
    pub end_time_milliseconds: u64,

    pub num_pols: u32,
    pub num_baselines: u64,
    pub integration_time_milliseconds: u64,

    pub num_fine_channels: u64,
    pub coarse_channels: Vec<u64>,
    // fine_channel_resolution and coarse_channel_bandwidth are in units of Hz.
    pub fine_channel_resolution: u64,
    pub coarse_channel_bandwidth: u64,

    pub metafits_filename: String,

    // `gpubox_batches` *must* be sorted appropriately. See
    // `gpubox::determine_gpubox_batches`. The order of the filenames
    // corresponds directly to other gpubox-related objects
    // (e.g. `gpubox_hdu_limits`).
    pub gpubox_batches: Vec<Vec<String>>,
    // We assume as little as possible about the data layout in the gpubox
    // files; here, a `BTreeMap` contains each unique UNIX time from every
    // gpubox, which is associated with another `BTreeMap`, associating each
    // gpubox number with a gpubox batch number and HDU index. The gpubox
    // number, batch number and HDU index are everything needed to find the
    // correct HDU out of all gpubox files.
    pub gpubox_time_map: BTreeMap<u64, BTreeMap<usize, (usize, usize)>>,

    // The number of bytes taken up by a scan in each gpubox file.
    pub scan_size: usize,
}

impl mwalibObsContext {
    /// From a path to a metafits file and paths to gpubox files, create a
    /// `mwalibObsContext`.
    ///
    /// The traits on the input parameters allow flexibility to input types.
    pub fn new<T: AsRef<Path> + AsRef<str> + ToString + fmt::Debug>(
        metafits: &T,
        gpuboxes: &[T],
    ) -> Result<mwalibObsContext, ErrorKind> {
        // Do the file stuff upfront. Check that at least one gpubox file is
        // present.
        if gpuboxes.is_empty() {
            return Err(ErrorKind::Custom(
                "gpubox / mwax fits files missing".to_string(),
            ));
        }

        let gpubox_batches = determine_gpubox_batches(&gpuboxes)?;

        // Open all the files.
        let mut gpubox_fptrs = Vec::with_capacity(gpubox_batches.len());
        let mut gpubox_time_map = BTreeMap::new();
        for (batch_num, batch) in gpubox_batches.iter().enumerate() {
            gpubox_fptrs.push(Vec::with_capacity(batch.len()));
            for gpubox_file in batch {
                let fptr = FitsFile::open(&gpubox_file)
                    .with_context(|| format!("Failed to open {:?}", gpubox_file))?;

                // Store the FITS file pointer for later.
                gpubox_fptrs[batch_num].push(fptr);
            }

            // Because of the way `fitsio` uses the mutable reference to the
            // file handle, it's easier to do another loop here than use `fptr`
            // above.
            for (gpubox_num, mut fptr) in gpubox_fptrs[batch_num].iter_mut().enumerate() {
                let time_map = map_unix_times_to_hdus(&mut fptr)?;
                for (time, hdu_index) in time_map {
                    // For the current `time`, check if it's in the map. If not,
                    // insert it and a new tree. Then check if `gpubox_num` is
                    // in the sub-map for this `time`, etc.
                    let new_time_tree = BTreeMap::new();
                    gpubox_time_map
                        .entry(time)
                        .or_insert(new_time_tree)
                        .entry(gpubox_num)
                        .or_insert((batch_num, hdu_index));
                }
            }
        }

        // Pull out observation details. Save the metafits HDU for faster
        // accesses.
        let mut metafits_fptr =
            FitsFile::open(&metafits).with_context(|| format!("Failed to open {:?}", metafits))?;
        let metafits_hdu = metafits_fptr
            .hdu(0)
            .with_context(|| format!("Failed to open HDU 1 for {:?}", metafits))?;

        let obsid = get_fits_key(&mut metafits_fptr, &metafits_hdu, "GPSTIME")
            .with_context(|| format!("Failed to read GPSTIME for {:?}", metafits))?;

        // Always assume that MWA data has four polarisations. Would this ever
        // not be true?
        let num_pols = 4;

        // Calculate the number of baselines. There are twice as many inputs as
        // there are antennas; halve that value.
        let num_inputs = get_fits_key::<u64>(&mut metafits_fptr, &metafits_hdu, "NINPUTS")
            .with_context(|| format!("Failed to read NINPUTS for {:?}", metafits))?
            / 2;
        let num_baselines = num_inputs / 2 * (num_inputs - 1);

        let integration_time_milliseconds =
            (get_fits_key::<f64>(&mut metafits_fptr, &metafits_hdu, "INTTIME")
                .with_context(|| format!("Failed to read INTTIME for {:?}", metafits))?
                * 1000.) as u64;

        // The coarse-channels string uses the FITS "CONTINUE" keywords. The
        // fitsio library for rust does not (appear) to handle CONTINUE keywords
        // at present, but the underlying fitsio-sys does, so we have to do FFI
        // directly.
        let coarse_channels = {
            // Read the long string.
            let (status, coarse_channels_string) =
                unsafe { get_fits_long_string(metafits_fptr.as_raw(), "CHANNELS") };
            if status != 0 {
                return Err(ErrorKind::Custom(
                    "mwalibObsContext::new: get_fits_long_string failed".to_string(),
                ));
            }

            coarse_channels_string
                .replace(&['\'', '&'][..], "")
                .split(',')
                .map(|s| s.parse().unwrap())
                .collect()
        };

        // Fine-channel resolution. The FINECHAN value in the metafits is in units
        // of kHz - make it Hz.
        let resolution = (get_fits_key::<f64>(&mut metafits_fptr, &metafits_hdu, "FINECHAN")
            .with_context(|| format!("Failed to read FINECHAN for {:?}", metafits))?
            * 1000.)
            .round() as u64;

        // Coarse-channel bandwidth.
        let bandwidth = (get_fits_key::<f64>(&mut metafits_fptr, &metafits_hdu, "BANDWDTH")
            .with_context(|| format!("Failed to read BANDWDTH for {:?}", metafits))?
            * 1e6)
            .round() as u64;

        // Determine the fine channels. For some reason, this isn't in the
        // metafits. Assume that this is the same for all gpubox files.
        let num_fine_channels = gpubox_fptrs[0][0]
            .hdu(1)
            .with_context(|| format!("Failed to open HDU 2 for {:?}", gpubox_batches[0][0]))?
            .read_key::<i64>(&mut gpubox_fptrs[0][0], "NAXIS2")
            .with_context(|| format!("Failed to read NAXIS2 for {:?}", gpubox_batches[0][0]))?
            as u64;
        // Go back to the first HDU.
        gpubox_fptrs[0][0]
            .hdu(0)
            .with_context(|| format!("Failed to open HDU 1 for {:?}", gpubox_batches[0][0]))?;

        // Populate the start and end times of the observation.
        let (start_time_milliseconds, end_time_milliseconds) = {
            let o = determine_obs_times(&gpubox_time_map)?;
            (o.start_millisec, o.end_millisec)
        };

        Ok(mwalibObsContext {
            obsid,
            start_time_milliseconds,
            end_time_milliseconds,
            num_pols,
            num_baselines,
            integration_time_milliseconds,
            num_fine_channels,
            coarse_channels,
            fine_channel_resolution: resolution,
            coarse_channel_bandwidth: bandwidth,
            metafits_filename: metafits.to_string(),
            gpubox_batches,
            gpubox_time_map,
            scan_size: (num_fine_channels * num_baselines * num_pols as u64) as usize,
        })
    }
}

impl fmt::Display for mwalibObsContext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(
            f,
            r#"mwalibObsContext (
    obsid:               {},
    obs UNIX start time: {} s,
    obs UNIX end time:   {} s,

    num baselines:    {},
    num pols:         {},

    num fine channels: {},
    coarse channels: {:?},
    fine channel resolution:  {} kHz,
    coarse channel bandwidth: {} MHz,

    metafits filename: {},
    gpubox batches: {:#?},
)"#,
            self.obsid,
            self.start_time_milliseconds as f64 / 1e3,
            self.end_time_milliseconds as f64 / 1e3,
            self.num_baselines,
            self.num_pols,
            self.num_fine_channels,
            self.coarse_channels,
            self.fine_channel_resolution as f64 / 1e3,
            self.coarse_channel_bandwidth as f64 / 1e6,
            self.metafits_filename,
            self.gpubox_batches,
        )
    }
}
