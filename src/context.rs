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

use crate::antenna::*;
use crate::coarse_channel::*;
use crate::convert::*;
use crate::fits_read::*;
use crate::gpubox::*;
use crate::rfinput::*;
use crate::timestep::*;
use crate::*;

/// Enum for all of the known variants of file format based on Correlator version
///
#[repr(C)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum CorrelatorVersion {
    /// MWAX correlator (v2.0)
    V2,
    /// MWA correlator (v1.0), having data files with "gpubox" and batch numbers in their names.
    Legacy,
    /// MWA correlator (v1.0), having data files without any batch numbers.
    OldLegacy,
}

/// Implements fmt::Display for CorrelatorVersion struct
///
/// # Arguments
///
/// * `f` - A fmt::Formatter
///
///
/// # Returns
///
/// * `fmt::Result` - Result of this method
///
///
impl fmt::Display for CorrelatorVersion {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                CorrelatorVersion::V2 => "v2 MWAX",
                CorrelatorVersion::Legacy => "v1 Legacy",
                CorrelatorVersion::OldLegacy => "v1 Legacy (no file indices)",
            }
        )
    }
}

/// `mwalib` observation context. This is used to transport data out of gpubox
/// files and display info on the observation.
///
/// The name is not following the rust convention of camel case, to make it look
/// more like a C library.
#[allow(non_camel_case_types)]
pub struct mwalibContext {
    /// Observation id
    pub obsid: u32,
    /// Version of the correlator format
    pub corr_version: CorrelatorVersion,
    /// "the velocity factor of electic fields in RG-6 like coax"    
    pub coax_v_factor: f64,
    /// The proper start of the observation (the time that is common to all
    /// provided gpubox files).
    pub start_unix_time_milliseconds: u64,
    /// `end_time_milliseconds` will reflect the start time of the *last* HDU it
    /// is derived from (i.e. `end_time_milliseconds` + integration time is the
    /// actual end time of the observation).
    pub end_unix_time_milliseconds: u64,
    /// Total duration of observation (based on gpubox files)
    pub duration_milliseconds: u64,
    /// Number of timesteps in the observation
    pub num_timesteps: usize,
    /// This is an array of all timesteps we have data for
    pub timesteps: Vec<mwalibTimeStep>,
    /// Total number of antennas (tiles) in the array
    pub num_antennas: usize,
    /// We also have just the antennas
    pub antennas: Vec<mwalibAntenna>,
    /// Number of baselines stored. This is autos plus cross correlations
    pub num_baselines: usize,
    /// Total number of rf_inputs (tiles * 2 pols X&Y)
    pub num_rf_inputs: usize,
    /// The Metafits defines an rf chain for antennas(tiles) * pol(X,Y)    
    pub rf_inputs: Vec<mwalibRFInput>,
    /// Number of antenna pols. e.g. X and Y
    pub num_antenna_pols: usize,
    /// Number of polarisation combinations in the visibilities e.g. XX,XY,YX,YY == 4
    pub num_visibility_pols: usize,
    /// Number of coarse channels after we've validated the input gpubox files
    pub num_coarse_channels: usize,
    /// Vector of coarse channel structs
    pub coarse_channels: Vec<mwalibCoarseChannel>,
    /// Correlator mode dump time
    pub integration_time_milliseconds: u64,
    /// Correlator fine_channel_resolution
    pub fine_channel_width_hz: u32,
    /// Total bandwidth of observation (of the coarse channels we have)
    pub observation_bandwidth_hz: u32,
    /// Bandwidth of each coarse channel
    pub coarse_channel_width_hz: u32,
    /// Number of fine channels in each coarse channel
    pub num_fine_channels_per_coarse: usize,
    /// Filename of the metafits we were given
    pub metafits_filename: String,

    /// `gpubox_batches` *must* be sorted appropriately. See
    /// `gpubox::determine_gpubox_batches`. The order of the filenames
    /// corresponds directly to other gpubox-related objects
    /// (e.g. `gpubox_hdu_limits`). Structured:
    /// `gpubox_batches[batch][filename]`.
    pub gpubox_batches: Vec<GPUBoxBatch>,

    /// We assume as little as possible about the data layout in the gpubox
    /// files; here, a `BTreeMap` contains each unique UNIX time from every
    /// gpubox, which is associated with another `BTreeMap`, associating each
    /// gpubox number with a gpubox batch number and HDU index. The gpubox
    /// number, batch number and HDU index are everything needed to find the
    /// correct HDU out of all gpubox files.
    pub gpubox_time_map: BTreeMap<u64, BTreeMap<usize, (usize, usize)>>,

    /// The number of bytes taken up by a scan/timestep in each gpubox file.
    pub num_timestep_coarse_channel_bytes: usize,
    /// The number of floats in each gpubox HDU.
    pub num_timestep_coarse_channel_floats: usize,
    /// This is the number of gpubox files *per batch*.
    pub num_gpubox_files: usize,
    /// A conversion table to optimise reading of legacy MWA HDUs
    pub legacy_conversion_table: Vec<mwalibLegacyConversionBaseline>,
}

impl mwalibContext {
    /// From a path to a metafits file and paths to gpubox files, create an `mwalibContext`.
    ///
    /// The traits on the input parameters allow flexibility to input types.
    ///
    /// # Arguments
    ///
    /// * `metafits` - filename of metafits file as a path or string.
    ///
    /// * `gpuboxes` - slice of filenames of gpubox files as paths or strings.
    ///
    ///
    /// # Returns
    ///
    /// * Result containing a populated mwalibContext object if Ok.
    ///
    ///
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

        // from MWA_Tools/CONV2UVFITS/convutils.h
        // Used to determine electrical lengths if EL_ not present in metafits for an rf_input
        let coax_v_factor: f64 = 1.204;

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

        let (mut gpubox_batches, corr_version, num_gpubox_files) =
            determine_gpubox_batches(&gpuboxes)?;

        let (gpubox_time_map, hdu_size) =
            gpubox::create_time_map(&mut gpubox_batches, corr_version)?;

        // Populate our array of timesteps
        // Create a vector of rf_input structs from the metafits
        let (timesteps, num_timesteps) = mwalibTimeStep::populate_timesteps(&gpubox_time_map)?;
        let num_rf_inputs = get_fits_key::<usize>(&mut metafits_fptr, &metafits_hdu, "NINPUTS")
            .with_context(|| format!("Failed to read NINPUTS for {:?}", metafits))?;

        // There are twice as many inputs as
        // there are antennas; halve that value.
        let num_antennas = num_rf_inputs / 2;

        // Create a vector of rf_input structs from the metafits
        let mut rf_inputs: Vec<mwalibRFInput> = mwalibRFInput::populate_rf_inputs(
            num_rf_inputs,
            &mut metafits_fptr,
            metafits_tile_table_hdu,
            coax_v_factor,
        )?;

        // Sort the rf_inputs back into the correct output order
        rf_inputs.sort_by_key(|k| k.subfile_order);

        // Now populate the antennas (note they need to be sorted by subfile_order)
        let antennas: Vec<mwalibAntenna> = mwalibAntenna::populate_antennas(&rf_inputs);

        let obsid = get_fits_key(&mut metafits_fptr, &metafits_hdu, "GPSTIME")
            .with_context(|| format!("Failed to read GPSTIME for {:?}", metafits))?;

        // Always assume that MWA antennas have 2 pols, therefore the data has four polarisations. Would this ever
        // not be true?
        let num_antenna_pols = 2;
        let num_visibility_pols = num_antenna_pols * num_antenna_pols;

        // `num_baselines` is the number of cross-correlations + the number of
        // auto-correlations.
        let num_baselines = (num_antennas / 2) * (num_antennas + 1);

        let integration_time_milliseconds: u64 =
            (get_fits_key::<f64>(&mut metafits_fptr, &metafits_hdu, "INTTIME")
                .with_context(|| format!("Failed to read INTTIME for {:?}", metafits))?
                * 1000.) as u64;
        // observation bandwidth (read from metafits in MHz)
        let metafits_observation_bandwidth_hz =
            (get_fits_key::<f64>(&mut metafits_fptr, &metafits_hdu, "BANDWDTH")
                .with_context(|| format!("Failed to read BANDWDTH for {:?}", metafits))?
                * 1e6)
                .round() as _;
        // Populate coarse channels
        let (coarse_channels, num_coarse_channels, coarse_channel_width_hz) =
            coarse_channel::mwalibCoarseChannel::populate_coarse_channels(
                &mut metafits_fptr,
                corr_version,
                metafits_observation_bandwidth_hz,
                &gpubox_time_map,
            )?;
        let observation_bandwidth_hz = (num_coarse_channels as u32) * coarse_channel_width_hz;

        // Fine-channel resolution. The FINECHAN value in the metafits is in units
        // of kHz - make it Hz.
        let fine_channel_width_hz =
            (get_fits_key::<f64>(&mut metafits_fptr, &metafits_hdu, "FINECHAN")
                .with_context(|| format!("Failed to read FINECHAN for {:?}", metafits))?
                * 1000.)
                .round() as _;

        // Determine the number of fine channels per coarse channel.
        let num_fine_channels_per_coarse =
            (coarse_channel_width_hz / fine_channel_width_hz) as usize;

        // Populate the start and end times of the observation.
        let (start_unix_time_milliseconds, end_unix_time_milliseconds, duration_milliseconds) = {
            let o = determine_obs_times(&gpubox_time_map, integration_time_milliseconds)?;
            (o.start_millisec, o.end_millisec, o.duration_millisec)
        };

        // Prepare the conversion array to convert legacy correlator format into mwax format
        // or just leave it empty if we're in any other format
        let legacy_conversion_table: Vec<mwalibLegacyConversionBaseline> = if corr_version
            == CorrelatorVersion::OldLegacy
            || corr_version == CorrelatorVersion::Legacy
        {
            convert::generate_conversion_array(&mut rf_inputs)
        } else {
            Vec::new()
        };

        // Sort the rf_inputs back into the correct output order
        rf_inputs.sort_by_key(|k| k.subfile_order);

        Ok(mwalibContext {
            corr_version,
            obsid,
            coax_v_factor,
            start_unix_time_milliseconds,
            end_unix_time_milliseconds,
            duration_milliseconds,
            num_timesteps,
            timesteps,
            num_antennas,
            antennas,
            num_baselines,
            num_rf_inputs,
            rf_inputs,
            integration_time_milliseconds,
            num_antenna_pols,
            num_visibility_pols,
            num_fine_channels_per_coarse,
            num_coarse_channels,
            coarse_channels,
            coarse_channel_width_hz,
            fine_channel_width_hz,
            observation_bandwidth_hz,
            metafits_filename: metafits.to_string(),
            gpubox_batches,
            gpubox_time_map,
            num_timestep_coarse_channel_bytes: hdu_size * 4,
            num_gpubox_files,
            num_timestep_coarse_channel_floats: hdu_size,
            legacy_conversion_table,
        })
    }

    /// Read a single timestep for a single coarse channel
    /// The output visibilities are in order:
    /// [baseline][frequency][pol][r][i]
    ///
    /// # Arguments
    ///
    /// * `timestep_index` - index within the timestep array for the desired timestep. This corresponds
    ///                      to the element within mwalibContext.timesteps.
    ///
    /// * `coarse_channel_index` - index within the coarse_channel array for the desired coarse channel. This corresponds
    ///                      to the element within mwalibContext.coarse_channels.
    ///
    ///
    /// # Returns
    ///
    /// * A Result containing vector of 32 bit floats containing the data in [baseline][frequency][pol][r][i] order, if Ok.
    ///
    ///
    pub fn read_by_baseline(
        &mut self,
        timestep_index: usize,
        coarse_channel_index: usize,
    ) -> Result<Vec<f32>, ErrorKind> {
        // Output buffer for read in data
        let output_buffer: Vec<f32>;

        // Prepare temporary buffer, if we are reading legacy correlator files
        let mut temp_buffer = if self.corr_version == CorrelatorVersion::OldLegacy
            || self.corr_version == CorrelatorVersion::Legacy
        {
            vec![
                0.;
                self.num_fine_channels_per_coarse
                    * self.num_visibility_pols
                    * self.num_baselines
                    * 2
            ]
        } else {
            Vec::new()
        };

        // Lookup the coarse channel we need
        let coarse_channel = self.coarse_channels[coarse_channel_index].gpubox_number;
        let (batch_index, hdu_index) =
            self.gpubox_time_map[&self.timesteps[timestep_index].unix_time_ms][&coarse_channel];

        let mut fptr = self.gpubox_batches[batch_index].gpubox_files[coarse_channel_index]
            .fptr
            .as_mut()
            .unwrap();
        let hdu = fptr.hdu(hdu_index)?;
        output_buffer = hdu.read_image(&mut fptr)?;
        // If legacy correlator, then convert the HDU into the correct output format
        if self.corr_version == CorrelatorVersion::OldLegacy
            || self.corr_version == CorrelatorVersion::Legacy
        {
            convert::convert_legacy_hdu(
                &self.legacy_conversion_table,
                &output_buffer,
                &mut temp_buffer,
                self.num_fine_channels_per_coarse,
            );

            Ok(temp_buffer)
        } else {
            Ok(output_buffer)
        }
    }
}

/// Implements fmt::Display for mwalibContext struct
///
/// # Arguments
///
/// * `f` - A fmt::Formatter
///
///
/// # Returns
///
/// * `fmt::Result` - Result of this method
///
///
impl fmt::Display for mwalibContext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // `size` is the number of floats (self.gpubox_hdu_size) multiplied by 4
        // bytes per float, divided by 1024^2 to get MiB.
        let size = (self.num_timestep_coarse_channel_floats * 4) as f64 / (1024 * 1024) as f64;
        writeln!(
            f,
            r#"mwalibContext (
    Correlator version:       {},

    obsid:                    {},
    obs UNIX start time:      {} s,
    obs UNIX end time:        {} s,
    obs duration:             {} s,
    num timesteps:            {},
    timesteps:                {:?},

    num antennas:             {},
    antennas:                 {:?},
    rf_inputs:                {:?},

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
            self.duration_milliseconds as f64 / 1e3,
            self.num_timesteps,
            self.timesteps,
            self.num_antennas,
            self.antennas,
            self.rf_inputs,
            self.num_baselines,
            self.num_antennas,
            self.num_baselines - self.num_antennas,
            self.num_antenna_pols,
            self.num_visibility_pols,
            self.observation_bandwidth_hz as f64 / 1e6,
            self.num_coarse_channels,
            self.coarse_channels,
            self.fine_channel_width_hz as f64 / 1e3,
            self.integration_time_milliseconds as f64 / 1e3,
            self.num_fine_channels_per_coarse,
            size,
            size * self.num_gpubox_files as f64,
            self.metafits_filename,
            self.gpubox_batches,
        )
    }
}
