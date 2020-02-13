// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

/*!
The main interface to MWA data.
 */

use std::collections::BTreeMap;
use std::fmt;
use std::path::*;

use fitsio::*;

use crate::fits_read::*;
use crate::gpubox::*;
use crate::*;

#[derive(Debug, PartialEq)]
pub enum CorrelatorVersion {
    /// New correlator data (a.k.a. MWAX).
    V2,
    /// MWA raw data files with "gpubox" and batch numbers in their names.
    Legacy,
    /// gpubox files without any batch numbers.
    OldLegacy,
}

impl fmt::Display for CorrelatorVersion {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                CorrelatorVersion::V2 => "V2 (MWAX)",
                CorrelatorVersion::Legacy => "Legacy",
                CorrelatorVersion::OldLegacy => "Legacy (no file indices)",
            }
        )
    }
}

#[allow(non_camel_case_types)]
pub struct mwalibAntenna {
    pub input: u32,
    pub antenna: u32,
    pub tile_id: u32,
    pub tile_name: String,
    pub pol: String,
    pub electrical_length: f64,
    pub north: f64,
    pub east: f64,
    pub height: f64,
}

impl fmt::Debug for mwalibAntenna {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.tile_name, self.pol)
    }
}

/// This is a struct for our coarse channels
#[allow(non_camel_case_types)]
pub struct mwalibCoarseChannel {
    // Correlator channel is 0 indexed
    pub correlator_channel_number: u16,

    // Receiver channel is 0-255 in the RRI recivers
    pub receiver_channel_number: u16,    

    pub channel_width_hz: u32,
    pub channel_start_hz: u32,
    pub channel_centre_hz: u32,
    pub channel_end_hz: u32,    
}

impl mwalibCoarseChannel {
    pub fn new(correlator_channel_number: u16, 
               receiver_channel_number: u16,
               channel_width_hz: u32, ) -> Result<mwalibCoarseChannel, ErrorKind> {
        let centre_chan_hz:u32 = (receiver_channel_number as u32) * channel_width_hz;

        Ok(mwalibCoarseChannel {
            correlator_channel_number, 
            receiver_channel_number,
            channel_width_hz,
            channel_centre_hz: centre_chan_hz,
            channel_start_hz: centre_chan_hz - (channel_width_hz / 2),
            channel_end_hz: centre_chan_hz + (channel_width_hz / 2),
        })
    }
}

impl fmt::Debug for mwalibCoarseChannel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "corr={} rec={} @ {:.3} MHz", self.correlator_channel_number, self.receiver_channel_number, self.channel_centre_hz as f32 / 1000000.)
    }
}

/// `mwalib` observation context. This is used to transport data out of gpubox
/// files and display info on the observation.
///
/// The name is not following the rust convention of camel case, to make it look
/// more like a C library.
#[allow(non_camel_case_types)]
pub struct mwalibContext {
    pub corr_version: CorrelatorVersion,

    pub obsid: u32,
    /// The proper start of the observation (the time that is common to all
    /// provided gpubox files).
    pub start_unix_time_milliseconds: u64,
    /// `end_time_milliseconds` will reflect the start time of the *last* HDU it
    /// is derived from (i.e. `end_time_milliseconds` + integration time is the
    /// actual end time of the observation).
    pub end_unix_time_milliseconds: u64,

    /// Track the UNIX time (in milliseconds) that will be read next.
    pub current_time_milliseconds: u64,

    /// Total number of antennas (tiles) in the array
    pub num_antennas: usize,
    
    /// The Metafits defines an rf chain for antennas(tiles) * pol(X,Y)    
    pub antennas: Vec<mwalibAntenna>,
    
    pub num_baselines: usize,
    pub integration_time_milliseconds: u64,

    /// Number of antenna pols. e.g. X and Y
    pub num_antenna_pols: usize,

    /// Number of polarisation combinations in the visibilities e.g. XX,XY,YX,YY == 4
    pub num_visibility_pols: usize,

    /// Number of fine channels in each coarse channel
    pub num_fine_channels: usize,

    pub num_coarse_channels: usize,
    pub coarse_channels: Vec<mwalibCoarseChannel>,

    /// fine_channel_resolution, coarse_channel_width and observation_bandwidth are in units of Hz.
    pub fine_channel_resolution_hz: u32,
    pub coarse_channel_width_hz: u32,
    pub observation_bandwidth_hz: u32,

    pub metafits_filename: String,

    /// `gpubox_batches` *must* be sorted appropriately. See
    /// `gpubox::determine_gpubox_batches`. The order of the filenames
    /// corresponds directly to other gpubox-related objects
    /// (e.g. `gpubox_hdu_limits`). Structured:
    /// `gpubox_batches[batch][filename]`.
    pub gpubox_batches: Vec<Vec<String>>,

    /// Keep gpubox file FITS file pointers open. Structured:
    /// `gpubox_fptrs[batch][fptr]`, in the same way `gpubox_batches` is laid
    /// out.
    pub gpubox_fptrs: Vec<Vec<FitsFile>>,

    /// We assume as little as possible about the data layout in the gpubox
    /// files; here, a `BTreeMap` contains each unique UNIX time from every
    /// gpubox, which is associated with another `BTreeMap`, associating each
    /// gpubox number with a gpubox batch number and HDU index. The gpubox
    /// number, batch number and HDU index are everything needed to find the
    /// correct HDU out of all gpubox files.
    pub gpubox_time_map: BTreeMap<u64, BTreeMap<usize, (usize, usize)>>,

    /// The number of bytes taken up by a scan in each gpubox file.
    pub scan_size: usize,

    /// These variables are provided for convenience, so a caller knows how to
    /// index the data buffer.
    pub num_data_scans: usize,
    /// This is the number of gpubox files *per batch*.
    pub num_gpubox_files: usize,
    /// The number of floats in each gpubox HDU.
    pub gpubox_hdu_size: usize,
}

impl mwalibContext {
    /// From a path to a metafits file and paths to gpubox files, create a
    /// `mwalibContext`.
    ///
    /// The traits on the input parameters allow flexibility to input types.
    pub fn new<T: AsRef<Path> + AsRef<str> + ToString + fmt::Debug>(
        metafits: &T,
        gpuboxes: &[T],
    ) -> Result<mwalibContext, ErrorKind> {
        // Do the file stuff upfront. Check that at least one gpubox file is
        // present.
        if gpuboxes.is_empty() {
            return Err(ErrorKind::Custom(
                "mwalibContext::new: gpubox / mwax fits files missing".to_string(),
            ));
        }

        let (gpubox_batches, corr_version) = determine_gpubox_batches(&gpuboxes)?;

        // Open all the files.
        let mut gpubox_fptrs = Vec::with_capacity(gpubox_batches.len());
        let mut gpubox_time_map = BTreeMap::new();
        // Keep track of the gpubox HDU size and the number of gpubox files.
        let mut size = 0;
        let mut num_gpubox_files = 0;
        for (batch_num, batch) in gpubox_batches.iter().enumerate() {
            num_gpubox_files = batch.len();
            gpubox_fptrs.push(Vec::with_capacity(batch.len()));
            for gpubox_file in batch {
                let mut fptr = FitsFile::open(&gpubox_file)
                    .with_context(|| format!("Failed to open {:?}", gpubox_file))?;

                let hdu = fptr
                    .hdu(0)
                    .with_context(|| format!("Failed to open HDU 1 of {:?}", gpubox_file))?;
                // New correlator files include a version - check that it is present.
                if corr_version == CorrelatorVersion::V2 {
                    let v = get_fits_key::<u8>(&mut fptr, &hdu, "CORR_VER").with_context(|| {
                        format!("Failed to read key CORR_VER from {:?}", gpubox_file)
                    })?;
                    if v != 2 {
                        return Err(ErrorKind::Custom(
                            "mwalibContext::new: MWAX gpubox file had a CORR_VER not equal to 2"
                                .to_string(),
                        ));
                    }
                }
                // Store the FITS file pointer for later.
                gpubox_fptrs[batch_num].push(fptr);
            }

            // Because of the way `fitsio` uses the mutable reference to the
            // file handle, it's easier to do another loop here than use `fptr`
            // above.
            for (gpubox_num, mut fptr) in gpubox_fptrs[batch_num].iter_mut().enumerate() {
                let time_map = map_unix_times_to_hdus(&mut fptr, &corr_version)?;
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

                // Determine the size of the gpubox HDU image. mwalib will panic
                // if this size is not consistent for all HDUs in all gpubox
                // files.
                let this_size = get_hdu_image_size(&mut fptr)?.iter().product();
                if size != 0 && size != this_size {
                    return Err(ErrorKind::Custom(
                        "mwalibBuffer::read: Error: HDU sizes in gpubox files are not equal"
                            .to_string(),
                    ));
                } else {
                    size = this_size;
                }
            }
        }

        // Pull out observation details. Save the metafits HDU for faster
        // accesses.        
        let mut metafits_fptr =
            FitsFile::open(&metafits).with_context(|| format!("Failed to open {:?}", metafits))?;
        let metafits_hdu = metafits_fptr
            .hdu(0)
            .with_context(|| format!("Failed to open HDU 1 (primary hdu) for {:?}", metafits))?;

        let metafits_tile_table_hdu = metafits_fptr
            .hdu(1)
            .with_context(|| format!("Failed to open HDU 2 (tiledata table) for {:?}", metafits))?;

        let num_inputs = get_fits_key::<usize>(&mut metafits_fptr, &metafits_hdu, "NINPUTS")
            .with_context(|| format!("Failed to read NINPUTS for {:?}", metafits))?;        

        // Create a vector of Antenna structs from the metafits
        let mut antennas:Vec<mwalibAntenna> = Vec::with_capacity(num_inputs);

        for input in 1..num_inputs {        
            // Note fits row numbers start at 1        
            let table_input = metafits_tile_table_hdu.read_cell_value(&mut metafits_fptr, "Input", input)
            .with_context(|| format!("Failed to read table for Input from {:?}.", metafits))?;
                        
            let table_antenna = metafits_tile_table_hdu.read_cell_value(&mut metafits_fptr, "Antenna", input)
            .with_context(|| format!("Failed to read table for Antenna from  {:?}", metafits))?;

            let table_tile_id = metafits_tile_table_hdu.read_cell_value(&mut metafits_fptr, "Tile", input)
            .with_context(|| format!("Failed to read table for Tile from  {:?}", metafits))?;

            let table_tile_name = metafits_tile_table_hdu.read_cell_value(&mut metafits_fptr, "TileName", input)
            .with_context(|| format!("Failed to read table for TileName from  {:?}", metafits))?;

            let table_pol = metafits_tile_table_hdu.read_cell_value(&mut metafits_fptr, "Pol", input)
            .with_context(|| format!("Failed to read table for Pol from  {:?}", metafits))?;

            // Length is stored as a string (no one knows why) starting with "EL_" the rest is a float so remove the prefix and get the float
            let table_electrical_length_desc:String = metafits_tile_table_hdu.read_cell_value(&mut metafits_fptr, "Length", input)
            .with_context(|| format!("Failed to read table for Length from  {:?}", metafits))?;            
            let table_electrical_length = table_electrical_length_desc.replace("EL_", "").parse().unwrap();

            let table_north = metafits_tile_table_hdu.read_cell_value(&mut metafits_fptr, "North", input)
            .with_context(|| format!("Failed to read table for North from  {:?}", metafits))?;

            let table_east = metafits_tile_table_hdu.read_cell_value(&mut metafits_fptr, "East", input)
            .with_context(|| format!("Failed to read table for East from  {:?}", metafits))?;

            let table_height = metafits_tile_table_hdu.read_cell_value(&mut metafits_fptr, "Height", input)
            .with_context(|| format!("Failed to read table for Height from {:?}", metafits))?;

            antennas.push(
                mwalibAntenna{
                    input: table_input,
                    antenna: table_antenna,
                    tile_id: table_tile_id,
                    tile_name: table_tile_name,
                    pol: table_pol,
                    electrical_length: table_electrical_length,
                    north: table_north,
                    east: table_east,
                    height: table_height,
            })
        }

        // Sort the Antenna vector by the "Antenna" column to get the actual order of tiles
        antennas.sort_by(|a, b| a.antenna.cmp(&b.antenna));
        
        let obsid = get_fits_key(&mut metafits_fptr, &metafits_hdu, "GPSTIME")
            .with_context(|| format!("Failed to read GPSTIME for {:?}", metafits))?;

        // Always assume that MWA antennas have 2 pols, therefore the data has four polarisations. Would this ever
        // not be true?
        let num_antenna_pols = 2;
        let num_visibility_pols = num_antenna_pols * num_antenna_pols;

        // Calculate the number of baselines. There are twice as many inputs as
        // there are antennas; halve that value.
        let num_antennas = num_inputs / 2;

        // `num_baselines` is the number of cross-correlations + the number of
        // auto-correlations.
        let num_baselines = (num_antennas / 2) * (num_antennas + 1);

        let integration_time_milliseconds:u64 =
            (get_fits_key::<f64>(&mut metafits_fptr, &metafits_hdu, "INTTIME")
                .with_context(|| format!("Failed to read INTTIME for {:?}", metafits))?
                * 1000.) as u64;        

        // The coarse-channels string uses the FITS "CONTINUE" keywords. The
        // fitsio library for rust does not (appear) to handle CONTINUE keywords
        // at present, but the underlying fitsio-sys does, so we have to do FFI
        // directly.
        let coarse_channel_vec:Vec<u16> = {
            // Read the long string.
            let (status, coarse_channels_string) =
                unsafe { get_fits_long_string(metafits_fptr.as_raw(), "CHANNELS") };
            if status != 0 {
                return Err(ErrorKind::Custom(
                    "mwalibContext::new: get_fits_long_string failed".to_string(),
                ));
            }

            coarse_channels_string
                .replace(&['\'', '&'][..], "")
                .split(',')
                .map(|s| s.parse().unwrap())
                .collect()
        };

        // How many coarse channels should there be (from the metafits)
        // NOTE: this will NOT always be equal to the number of gpubox files we get
        let num_coarse_channels = coarse_channel_vec.len();

        // observation bandwidth (read from metafits in MHz)
        let observation_bandwidth_hz = (get_fits_key::<f64>(&mut metafits_fptr, &metafits_hdu, "BANDWDTH")
            .with_context(|| format!("Failed to read BANDWDTH for {:?}", metafits))?
            * 1e6)
            .round() as _;

        // determine coarse channel width
        let coarse_channel_width_hz = observation_bandwidth_hz / (num_coarse_channels as u32);

        // Initialise the coarse channel vector of structs
        let mut coarse_channels:Vec<mwalibCoarseChannel> = Vec::with_capacity(num_coarse_channels);
        let mut i:usize = 0;

        for rec_channel_number in &coarse_channel_vec {            
            let mut corr_channel_number = i;

            if corr_version == CorrelatorVersion::Legacy ||
               corr_version == CorrelatorVersion::OldLegacy {
                // Legacy and Old Legacy: if receiver channel number is >127 then the order is reversed                
                if rec_channel_number > &127 {
                    corr_channel_number  = (num_coarse_channels -1) - i;
                }                
            }                         

            coarse_channels.push(
                mwalibCoarseChannel::new(corr_channel_number as u16, 
                                         *rec_channel_number,
                                         coarse_channel_width_hz).unwrap()
            );

            i = i + 1;
        }

        // Now sort the coarse channels by receiver channel number
        coarse_channels.sort_by(|a, b| a.receiver_channel_number.cmp(&b.receiver_channel_number));

        // Fine-channel resolution. The FINECHAN value in the metafits is in units
        // of kHz - make it Hz.
        let resolution = (get_fits_key::<f64>(&mut metafits_fptr, &metafits_hdu, "FINECHAN")
            .with_context(|| format!("Failed to read FINECHAN for {:?}", metafits))?
            * 1000.)
            .round() as _;        

        // Determine the fine channels. For some reason, this isn't in the
        // metafits. Assume that this is the same for all gpubox files.
        let num_fine_channels = {
            let fptr = &mut gpubox_fptrs[0][0];
            let hdu = fptr
                .hdu(1)
                .with_context(|| format!("Failed to open HDU 2 for {:?}", gpubox_batches[0][0]))?;

            if corr_version == CorrelatorVersion::V2 {
                get_fits_key::<usize>(&mut gpubox_fptrs[0][0], &hdu, "NAXIS1").with_context(
                    || format!("Failed to read NAXIS1 for {:?}", gpubox_batches[0][0]),
                )? / num_visibility_pols / 2
            } else {
                get_fits_key(&mut gpubox_fptrs[0][0], &hdu, "NAXIS2").with_context(|| {
                    format!("Failed to read NAXIS2 for {:?}", gpubox_batches[0][0])
                })?
            }
        };

        // Populate the start and end times of the observation.
        let (start_unix_time_milliseconds, end_unix_time_milliseconds) = {
            let o = determine_obs_times(&gpubox_time_map)?;
            (o.start_millisec, o.end_millisec)
        };

        Ok(mwalibContext {
            corr_version,
            obsid,
            start_unix_time_milliseconds,
            end_unix_time_milliseconds,
            current_time_milliseconds: start_unix_time_milliseconds,
            num_antennas, 
            antennas,           
            num_baselines,
            integration_time_milliseconds,
            num_antenna_pols,
            num_visibility_pols,
            num_fine_channels,
            num_coarse_channels,
            coarse_channel_width_hz,
            coarse_channels,
            fine_channel_resolution_hz: resolution,
            observation_bandwidth_hz,
            metafits_filename: metafits.to_string(),
            gpubox_batches,
            gpubox_fptrs,
            gpubox_time_map,
            scan_size: num_fine_channels * num_baselines * num_visibility_pols,
            // Set `num_data_scans` to 1 here. The caller will specify how many
            // scans to read in `mwalibContext::read` function.
            num_data_scans: 1,
            num_gpubox_files,
            gpubox_hdu_size: size,
        })
    }

    /// The output `buffer` is structured: `buffer[scan][gpubox_index][data]`.
    pub fn read(&mut self, num_scans: usize) -> Result<Vec<Vec<Vec<f32>>>, ErrorKind> {
        // Is there enough data left to fit into the total number of scans? If
        // not, we need to resize `buffer`.
        let ct = self.current_time_milliseconds as i64;
        let it = self.integration_time_milliseconds as i64;
        let et = self.end_unix_time_milliseconds as i64;
        // The end time is inclusive; need to add the integration time to get
        // the last scan.
        let new_num_scans = ((et - ct + it) as f64 / it as f64) as i64;

        if new_num_scans < 0 {
            return Err(ErrorKind::Custom("mwalibBuffer::read: A negative number for `new_num_scans` was calculated; this should only happen if something has manually changed the timings.".to_string()));
        };

        // Compare the input requested number of scans against `new_num_scans`
        // and take the smaller of the two. Keep the result in the struct.
        self.num_data_scans = num_scans.min(new_num_scans as usize);
        // Completely reset the internal data buffer.
        let mut data = vec![vec![vec![]; self.num_gpubox_files]; self.num_data_scans];

        for scan in &mut data {
            for gpubox_index in 0..self.num_gpubox_files {
                let (batch_index, hdu_index) =
                    self.gpubox_time_map[&self.current_time_milliseconds][&gpubox_index];
                let mut fptr = &mut self.gpubox_fptrs[batch_index][gpubox_index];
                let hdu = fptr.hdu(hdu_index)?;
                scan[gpubox_index] = hdu.read_image(&mut fptr)?;

                // We expect *all* HDUs to have the same number of floats. Error
                // if this is not true.
                if scan[gpubox_index].len() != self.gpubox_hdu_size {
                    return Err(ErrorKind::Custom(
                        "mwalibBuffer::read: Error: HDU sizes in gpubox files are not equal"
                            .to_string(),
                    ));
                }
            }
            self.current_time_milliseconds += self.integration_time_milliseconds as u64;
        }

        Ok(data)
    }
}

impl fmt::Display for mwalibContext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // `size` is the number of floats (self.gpubox_hdu_size) multiplied by 4
        // bytes per float, divided by 1024^2 to get MiB.
        let size = (self.gpubox_hdu_size * 4) as f64 / (1024 * 1024) as f64;
        writeln!(
            f,
            r#"mwalibContext (
    Correlator version:       {},

    obsid:                    {},
    obs UNIX start time:      {} s,
    obs UNIX end time:        {} s,

    num antennas:             {},
    tiles:                    {:?},

    num baselines:            {},
    num auto-correlations:    {},
    num cross-correlations:   {},

    num antenna pols:         {},
    num visibility pols:      {},
        
    observation bandwidth:    {} MHz,
    num coarse channels,      {},
    coarse channels:          {:?},

    Correlator Mode:    
    fine channel resolution:  {} kHz,
    integration time:         {:.2} s
    num fine channels/coarse: {},

    gpubox HDU size:          {} MiB,
    Memory usage per scan:    {} MiB,

    metafits filename:        {},
    gpubox batches:           {:#?},
)"#,
            self.corr_version,

            self.obsid,
            self.start_unix_time_milliseconds as f64 / 1e3,
            self.end_unix_time_milliseconds as f64 / 1e3,
            
            self.num_antennas,
            self.antennas,
            self.num_baselines,
            self.num_antennas,
            self.num_baselines - self.num_antennas,
            
            self.num_antenna_pols,
            self.num_visibility_pols,
            
            self.observation_bandwidth_hz as f64 / 1e6,
            self.num_coarse_channels,
            self.coarse_channels,            
            
            self.fine_channel_resolution_hz as f64 / 1e3,
            self.integration_time_milliseconds as f64 / 1e3,           
            self.num_fine_channels,             
                        
            size,
            size * self.num_gpubox_files as f64,
            self.metafits_filename,
            self.gpubox_batches,
        )
    }
}
