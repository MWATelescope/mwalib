/*!
This module contains rust-internal mwalib types.

Struct names are not following the rust convention of camel case, to make them
more look more like a C library.
 */

use std::fmt;
use std::path::*;

use anyhow::*;
use fitsio::*;

use crate::fits_read::*;
use crate::gpubox::*;
use crate::*;

#[allow(non_camel_case_types)]
pub struct mwalibObsContext {
    pub obsid: u32,
    pub start_time_milliseconds: u64,
    pub end_time_milliseconds: u64,

    // num_integrations only considers data between the start and end times!
    pub num_integrations: u32,
    pub num_baselines: u32,
    pub num_pols: u32,

    pub num_fine_channels: u32,
    pub coarse_channels: Vec<u32>,
    // fine_channel_resolution and coarse_channel_bandwidth are in units of Hz.
    pub fine_channel_resolution: u64,
    pub coarse_channel_bandwidth: u64,

    pub metafits_filename: String,
    pub metafits_fptr: FitsFile,

    // Elements of gpubox_batches are expected to be in the same order as
    // gpubox_ptrs. This concept applies all over mwalib.
    pub gpubox_batches: Vec<Vec<String>>,
    pub gpubox_fptrs: Vec<Vec<FitsFile>>,
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
        for (i, batch) in gpubox_batches.iter().enumerate() {
            gpubox_fptrs.push(Vec::with_capacity(batch.len()));
            for g in batch {
                let fptr = FitsFile::open(&g).with_context(|| format!("Failed to open {:?}", g))?;
                gpubox_fptrs[i].push(fptr);
            }
        }
        let mut metafits_fptr =
            FitsFile::open(&metafits).with_context(|| format!("Failed to open {:?}", metafits))?;

        // Pull out values. Save the metafits HDU for faster accesses.
        let metafits_hdu = metafits_fptr
            .hdu(0)
            .with_context(|| format!("Failed to open HDU 1 for {:?}", metafits))?;

        let obsid = get_fits_key::<u32>(&mut metafits_fptr, &metafits_hdu, "GPSTIME")
            .with_context(|| format!("Failed to read GPSTIME for {:?}", metafits))?;

        // Calculate the number of baselines. There are twice as many inputs as
        // there are antennas; halve that value.
        let num_inputs = get_fits_key::<u32>(&mut metafits_fptr, &metafits_hdu, "NINPUTS")
            .with_context(|| format!("Failed to read NINPUTS for {:?}", metafits))?
            / 2;
        let num_baselines = num_inputs / 2 * (num_inputs - 1);

        // The coarse-channels string uses the FITS "CONTINUE" keywords. The
        // fitsio library for rust does not (appear) to handle CONTINUE keywords
        // at present, but the underlying fitsio-sys does, so we have to do FFI
        // directly.
        let coarse_channels = unsafe {
            // Open the metafits file.
            let (status, fptr) = open_fits_ffi(&metafits.to_string());
            if status != 0 {
                return Err(ErrorKind::Custom(
                    "mwalibObsContext::new: open_fits_ffi failed".to_string(),
                ));
            }

            // Read the long string.
            let (status, coarse_channels_string) = get_fits_long_string(fptr, "CHANNELS");
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

        // Determine the fine channels. For some reason, this isn't in the metafits.
        let num_fine_channels = gpubox_fptrs[0][0]
            .hdu(1)
            .with_context(|| format!("Failed to open HDU 2 for {:?}", gpubox_batches[0][0]))?
            .read_key::<i64>(&mut gpubox_fptrs[0][0], "NAXIS2")
            .with_context(|| format!("Failed to read NAXIS2 for {:?}", gpubox_batches[0][0]))?
            as u32;
        // Go back to the first HDU.
        gpubox_fptrs[0][0]
            .hdu(0)
            .with_context(|| format!("Failed to open HDU 1 for {:?}", gpubox_batches[0][0]))?;

        // Populate the start and end times of the observation.
        let (start_time_milliseconds, end_time_milliseconds) = {
            let o = determine_obs_times(&mut gpubox_fptrs)?;
            (o.start_millisec, o.end_millisec)
        };

        /* char *antenna = (char *)malloc(sizeof(char) * 1024); */
        /* if (get_fits_string_value(obs->metafits_ptr, "Antenna", antenna, errorMessage) != EXIT_SUCCESS) { */
        /*     return EXIT_FAILURE; */
        /* } */
        /* printf("antenna: %s\n", antenna); */

        let num_integrations = 0;

        Ok(mwalibObsContext {
            obsid,
            start_time_milliseconds,
            end_time_milliseconds,
            num_integrations,
            num_baselines,
            // Always assume that MWA data has four polarisations. Would this ever
            // not be true?
            num_pols: 4,
            num_fine_channels,
            coarse_channels,
            fine_channel_resolution: resolution,
            coarse_channel_bandwidth: bandwidth,
            metafits_filename: metafits.to_string(),
            metafits_fptr,
            gpubox_batches,
            gpubox_fptrs,
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

    num integrations: {},
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
            self.num_integrations,
            self.num_baselines,
            self.num_pols,
            self.num_fine_channels,
            self.coarse_channels,
            self.fine_channel_resolution as f64 / 1e3,
            self.coarse_channel_bandwidth as f64 / 1e6,
            self.metafits_filename,
            self.gpubox_batches
        )
    }
}
