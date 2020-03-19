// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

/*!
The main interface to MWA data.
 */
use chrono::{DateTime, FixedOffset};
use fitsio::*;
use std::collections::BTreeMap;
use std::fmt;
use std::path::*;

use crate::antenna::*;
use crate::coarse_channel::*;
use crate::convert::*;
use crate::fits_read::*;
use crate::gpubox::*;
use crate::misc::*;
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
    // Instrument details
    // Lat
    pub mwa_latitude_radians: f64,
    // Long
    pub mwa_longitude_radians: f64,
    // altitude
    pub mwa_altitude_metres: f64,

    // "the velocity factor of electic fields in RG-6 like coax"
    pub coax_v_factor: f64,
    // Observation id
    pub obsid: u32,
    // Scheduled start (UTC) of observation
    pub scheduled_start_utc: DateTime<FixedOffset>,
    // Scheduled start (MJD) of observation
    pub scheduled_start_mjd: f64,
    // Scheduled duration of observation
    pub scheduled_duration_milliseconds: u64,
    // RA tile pointing
    pub ra_tile_pointing_degrees: f64,
    // DEC tile pointing
    pub dec_tile_pointing_degrees: f64,
    // RA phase centre
    pub ra_phase_center_degrees: Option<f64>,
    // DEC phase centre
    pub dec_phase_center_degrees: Option<f64>,
    // AZIMUTH
    pub azimuth_degrees: f64,
    // ALTITUDE
    pub altitude_degrees: f64,
    // Altitude of Sun
    pub sun_altitude_degrees: f64,
    // Distance from pointing center to Sun
    pub sun_distance_degrees: f64,
    // Distance from pointing center to the Moon
    pub moon_distance_degrees: f64,
    // Distance from pointing center to Jupiter
    pub jupiter_distance_degrees: f64,
    // Local Sidereal Time
    pub lst_degrees: f64,
    // Hour Angle of pointing center
    pub hour_angle_string: String,
    // GRIDNAME
    pub grid_name: String,
    // GRIDNUM
    pub grid_number: i32,
    // CREATOR
    pub creator: String,
    // PROJECT
    pub project_id: String,
    // Observation name
    pub observation_name: String,
    // MWA observation mode
    pub mode: String,
    // TODO: RECVRS    // Array of receiver numbers (this tells us how many receivers too)
    // TODO: DELAYS    // Array of delays
    // TODO: ATTEN_DB  // global analogue attenuation, in dB
    pub global_analogue_attenuation_db: f64,
    // TODO: QUACKTIM  // Seconds of bad data after observation starts
    pub quack_time_duration_milliseconds: u64,
    // TODO: GOODTIME  // OBSID+QUACKTIM as Unix timestamp (first good timestep)
    pub good_time_unix_milliseconds: u64,
    /// Version of the correlator format
    pub corr_version: CorrelatorVersion,
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
    ) -> Result<Self, ErrorKind> {
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

        let mwa_latitude_radians: f64 = dms_to_degrees(-26, 42, 11.94986).to_radians(); // -26d42m11.94986s
        let mwa_longitude_radians: f64 = dms_to_degrees(116, 40, 14.93485).to_radians(); // 116d40m14.93485s
        let mwa_altitude_metres: f64 = 377.827;

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
        let (timesteps, num_timesteps) = mwalibTimeStep::populate_timesteps(&gpubox_time_map);
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

        // Populate obsid
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
        // Start= start of first timestep
        // End  = start of last timestep + integration time
        let (start_unix_time_milliseconds, end_unix_time_milliseconds, duration_milliseconds) = {
            let o = determine_obs_times(&gpubox_time_map, integration_time_milliseconds)?;
            (o.start_millisec, o.end_millisec, o.duration_millisec)
        };

        // populate lots of useful metadata
        let scheduled_start_utc_string =
            get_fits_key_string(&mut metafits_fptr, &metafits_hdu, "DATE-OBS")
                .with_context(|| format!("Failed to read DATE-OBS for {:?}", metafits))?;

        let scheduled_start_utc_string_with_offset = scheduled_start_utc_string + "+00:00";

        let scheduled_start_utc =
            DateTime::parse_from_rfc3339(&scheduled_start_utc_string_with_offset)
                .expect("Unable to parse DATE-OBS into a date time");
        let scheduled_start_mjd: f64 = get_fits_key(&mut metafits_fptr, &metafits_hdu, "MJD")
            .with_context(|| format!("Failed to read MJD for {:?}", metafits))?;
        let scheduled_duration_milliseconds: u64 =
            get_fits_key::<u64>(&mut metafits_fptr, &metafits_hdu, "EXPOSURE")
                .with_context(|| format!("Failed to read EXPOSURE for {:?}", metafits))?
                * 1000;
        let ra_tile_pointing_degrees: f64 =
            get_fits_key(&mut metafits_fptr, &metafits_hdu, "RA")
                .with_context(|| format!("Failed to read RA for {:?}", metafits))?;
        let dec_tile_pointing_degrees: f64 = get_fits_key(&mut metafits_fptr, &metafits_hdu, "DEC")
            .with_context(|| format!("Failed to read DEC for {:?}", metafits))?;
        let ra_phase_center_degrees: Option<f64> =
            match get_fits_key(&mut metafits_fptr, &metafits_hdu, "RAPHASE")
                .with_context(|| format!("Failed to read RAPHASE for {:?}", metafits))
            {
                Ok(v) => Some(v),
                Err(_) => None,
            };
        let dec_phase_center_degrees: Option<f64> =
            match get_fits_key(&mut metafits_fptr, &metafits_hdu, "DECPHASE")
                .with_context(|| format!("Failed to read DECPHASE for {:?}", metafits))
            {
                Ok(v) => Some(v),
                Err(_) => None,
            };
        let azimuth_degrees: f64 = get_fits_key(&mut metafits_fptr, &metafits_hdu, "AZIMUTH")
            .with_context(|| format!("Failed to read AZIMUTH for {:?}", metafits))?;
        let altitude_degrees: f64 = get_fits_key(&mut metafits_fptr, &metafits_hdu, "ALTITUDE")
            .with_context(|| format!("Failed to read ALTITUDE for {:?}", metafits))?;
        let sun_altitude_degrees: f64 = get_fits_key(&mut metafits_fptr, &metafits_hdu, "SUN-ALT")
            .with_context(|| format!("Failed to read SUN-ALT for {:?}", metafits))?;
        let sun_distance_degrees: f64 = get_fits_key(&mut metafits_fptr, &metafits_hdu, "SUN-DIST")
            .with_context(|| format!("Failed to read SUN-DIST for {:?}", metafits))?;
        let moon_distance_degrees: f64 =
            get_fits_key(&mut metafits_fptr, &metafits_hdu, "MOONDIST")
                .with_context(|| format!("Failed to read MOONDIST for {:?}", metafits))?;
        let jupiter_distance_degrees: f64 =
            get_fits_key(&mut metafits_fptr, &metafits_hdu, "JUP-DIST")
                .with_context(|| format!("Failed to read JUP-DIST for {:?}", metafits))?;
        let lst_degrees: f64 = get_fits_key(&mut metafits_fptr, &metafits_hdu, "LST")
            .with_context(|| format!("Failed to read LST for {:?}", metafits))?;
        let hour_angle_string = get_fits_key_string(&mut metafits_fptr, &metafits_hdu, "HA")
            .with_context(|| format!("Failed to read HA for {:?}", metafits))?;
        let grid_name = get_fits_key_string(&mut metafits_fptr, &metafits_hdu, "GRIDNAME")
            .with_context(|| format!("Failed to read GRIDNAME for {:?}", metafits))?;
        let grid_number = get_fits_key(&mut metafits_fptr, &metafits_hdu, "GRIDNUM")
            .with_context(|| format!("Failed to read GRIDNUM for {:?}", metafits))?;
        let creator = get_fits_key_string(&mut metafits_fptr, &metafits_hdu, "CREATOR")
            .with_context(|| format!("Failed to read CREATOR for {:?}", metafits))?;
        let project_id = get_fits_key_string(&mut metafits_fptr, &metafits_hdu, "PROJECT")
            .with_context(|| format!("Failed to read PROJECT for {:?}", metafits))?;
        let observation_name =
            get_fits_key_string(&mut metafits_fptr, &metafits_hdu, "FILENAME")
                .with_context(|| format!("Failed to read FILENAME for {:?}", metafits))?;
        let mode = get_fits_key_string(&mut metafits_fptr, &metafits_hdu, "MODE")
            .with_context(|| format!("Failed to read MODE for {:?}", metafits))?;
        let receivers_string = get_fits_key_string(&mut metafits_fptr, &metafits_hdu, "RECVRS")
            .with_context(|| format!("Failed to read RECVRS for {:?}", metafits))?;
        let delays_string = get_fits_key_string(&mut metafits_fptr, &metafits_hdu, "DELAYS")
            .with_context(|| format!("Failed to read DELAYS for {:?}", metafits))?;
        let global_analogue_attenuation_db: f64 =
            get_fits_key(&mut metafits_fptr, &metafits_hdu, "ATTEN_DB")
                .with_context(|| format!("Failed to read ATTEN_DB for {:?}", metafits))?;
        let quack_time_duration_milliseconds: u64 =
            (get_fits_key::<f64>(&mut metafits_fptr, &metafits_hdu, "QUACKTIM")
                .with_context(|| format!("Failed to read QUACKTIM for {:?}", metafits))?
                * 1000.)
                .round() as _;
        let good_time_unix_milliseconds: u64 =
            (get_fits_key::<f64>(&mut metafits_fptr, &metafits_hdu, "GOODTIME")
                .with_context(|| format!("Failed to read GOODTIME for {:?}", metafits))?
                * 1000.)
                .round() as _;

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
            coax_v_factor,
            mwa_latitude_radians,
            mwa_longitude_radians,
            mwa_altitude_metres,
            corr_version,
            obsid,
            scheduled_start_utc,
            scheduled_start_mjd,
            scheduled_duration_milliseconds,
            ra_tile_pointing_degrees,
            dec_tile_pointing_degrees,
            ra_phase_center_degrees,
            dec_phase_center_degrees,
            azimuth_degrees,
            altitude_degrees,
            sun_altitude_degrees,
            sun_distance_degrees,
            moon_distance_degrees,
            jupiter_distance_degrees,
            lst_degrees,
            hour_angle_string,
            grid_name,
            grid_number,
            creator,
            project_id,
            observation_name,
            mode,
            global_analogue_attenuation_db,
            quack_time_duration_milliseconds,
            good_time_unix_milliseconds,
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
            num_gpubox_files,
            num_timestep_coarse_channel_bytes: hdu_size * 4,
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
    ) -> Result<Vec<f32>, fitsio::errors::Error> {
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
            convert::convert_legacy_hdu_to_mwax_baseline_order(
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

    /// Read a single timestep for a single coarse channel
    /// The output visibilities are in order:
    /// [frequency][baseline][pol][r][i]
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
    /// * A Result containing vector of 32 bit floats containing the data in [frequency][baseline][pol][r][i] order, if Ok.
    ///
    ///
    pub fn read_by_frequency(
        &mut self,
        timestep_index: usize,
        coarse_channel_index: usize,
    ) -> Result<Vec<f32>, fitsio::errors::Error> {
        // Output buffer for read in data
        let output_buffer: Vec<f32>;

        // Prepare temporary buffer, if we are reading legacy correlator files
        let mut temp_buffer = vec![
            0.;
            self.num_fine_channels_per_coarse
                * self.num_visibility_pols
                * self.num_baselines
                * 2
        ];

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
            convert::convert_legacy_hdu_to_mwax_frequency_order(
                &self.legacy_conversion_table,
                &output_buffer,
                &mut temp_buffer,
                self.num_fine_channels_per_coarse,
            );

            Ok(temp_buffer)
        } else {
            // Do conversion for mwax (it is in baseline order, we want it in freq order)
            convert::convert_mwax_hdu_to_frequency_order(
                &output_buffer,
                &mut temp_buffer,
                self.num_baselines,
                self.num_fine_channels_per_coarse,
                self.num_visibility_pols,
            );

            Ok(temp_buffer)
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

    MWA latitude:             {} degrees,
    MWA longitude:            {} degrees
    MWA altitude:             {} m,

    obsid:                    {},

    Creator:                  {},
    Project ID:               {},
    Observation Name:         {},    
    Global attenuation:       {} dB,

    Scheduled start (utc)     {},
    Scheduled start (MJD)     {},
    Scheduled duration        {} s,
    Actual UNIX start time:   {},
    Actual UNIX end time:     {},
    Actual duration:          {} s,
    Quack time:               {} s,
    Good UNIX start time:     {},

    R.A. (tile_pointing):     {} degrees,
    Dec. (tile_pointing):     {} degrees,
    R.A. (phase center):      {:?} degrees,
    Dec. (phase center):      {:?} degrees,
    Azimuth:                  {} degrees,
    Altitude:                 {} degrees,
    Sun altitude:             {} degrees,
    Sun distance:             {} degrees,
    Moon distance:            {} degrees,
    Jupiter distance:         {} degrees,
    LST:                      {} degrees,
    Hour angle:               {} degrees,
    Grid name:                {},
    Grid number:              {},    
    
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
    Mode:                     {},
    fine channel resolution:  {} kHz,
    integration time:         {:.2} s
    num fine channels/coarse: {},

    gpubox HDU size:          {} MiB,
    Memory usage per scan:    {} MiB,

    metafits filename:        {},
    gpubox batches:           {:#?},
)"#,
            self.corr_version,
            self.mwa_latitude_radians.to_degrees(),
            self.mwa_longitude_radians.to_degrees(),
            self.mwa_altitude_metres,
            self.obsid,
            self.creator,
            self.project_id,
            self.observation_name,
            self.global_analogue_attenuation_db,
            self.scheduled_start_utc,
            self.scheduled_start_mjd,
            self.scheduled_duration_milliseconds as f64 / 1e3,
            self.start_unix_time_milliseconds as f64 / 1e3,
            self.end_unix_time_milliseconds as f64 / 1e3,
            self.duration_milliseconds as f64 / 1e3,
            self.quack_time_duration_milliseconds as f64 / 1e3,
            self.good_time_unix_milliseconds as f64 / 1e3,
            self.ra_tile_pointing_degrees,
            self.dec_tile_pointing_degrees,
            self.ra_phase_center_degrees,
            self.dec_phase_center_degrees,
            self.azimuth_degrees,
            self.altitude_degrees,
            self.sun_altitude_degrees,
            self.sun_distance_degrees,
            self.moon_distance_degrees,
            self.jupiter_distance_degrees,
            self.lst_degrees,
            self.hour_angle_string,
            self.grid_name,
            self.grid_number,
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
            self.mode,
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_mwax_read() {
        // Open the test mwax file
        // a) directly using Fits  (data will be ordered [baseline][freq][pol][r][i])
        // b) using mwalib (by baseline) (data will be ordered the same as the raw fits file)
        // Then check b) is the same as a)
        let mwax_metafits_filename = "test_files/1244973688_1_timestep/1244973688.metafits";
        let mwax_filename =
            "test_files/1244973688_1_timestep/1244973688_20190619100110_ch114_000.fits";

        //
        // Read the mwax file using FITS
        //
        let mut fptr = FitsFile::open(&mwax_filename).unwrap();
        let fits_hdu = fptr.hdu(1).unwrap();

        // Read data from fits hdu into vector
        let fits_hdu_data: Vec<f32> = fits_hdu.read_image(&mut fptr).unwrap();

        //
        // Read the mwax file by frequency using mwalib
        //
        // Open a context and load in a test metafits and gpubox file
        let metafits: String = String::from(mwax_metafits_filename);
        let gpuboxfiles: Vec<String> = vec![String::from(mwax_filename)];
        let mut context =
            mwalibContext::new(&metafits, &gpuboxfiles).expect("Failed to create mwalibContext");

        // Read and convert first HDU
        let mwalib_hdu_data: Vec<f32> = context.read_by_baseline(0, 0).expect("Error!");

        // First assert that the data vectors are the same size
        assert_eq!(fits_hdu_data.len(), mwalib_hdu_data.len());
        // Check this block of floats matches
        assert_eq!(fits_hdu_data, mwalib_hdu_data);
    }
}
