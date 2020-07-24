// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

/*!
The main interface to MWA data.
 */
use std::collections::BTreeMap;
use std::fmt;

use chrono::{DateTime, Duration, FixedOffset};

use crate::antenna::*;
use crate::baseline::*;
use crate::coarse_channel::*;
use crate::convert::*;
use crate::gpubox::*;
use crate::misc::*;
use crate::rfinput::*;
use crate::timestep::*;
use crate::visibility_pol::*;
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
#[cfg(not(tarpaulin_include))]
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
    /// Latitude of centre point of MWA in raidans
    pub mwa_latitude_radians: f64,
    /// Longitude of centre point of MWA in raidans
    pub mwa_longitude_radians: f64,
    /// Altitude of centre poing of MWA in metres
    pub mwa_altitude_metres: f64,
    /// the velocity factor of electic fields in RG-6 like coax
    pub coax_v_factor: f64,
    /// Observation id
    pub obsid: u32,
    /// Scheduled start (gps time) of observation
    pub scheduled_start_gpstime_milliseconds: u64,
    /// Scheduled end (gps time) of observation
    pub scheduled_end_gpstime_milliseconds: u64,
    /// Scheduled start (UNIX time) of observation
    pub scheduled_start_unix_time_milliseconds: u64,
    /// Scheduled end (UNIX time) of observation
    pub scheduled_end_unix_time_milliseconds: u64,
    /// Scheduled start (UTC) of observation
    pub scheduled_start_utc: DateTime<FixedOffset>,
    /// Scheduled end (UTC) of observation
    pub scheduled_end_utc: DateTime<FixedOffset>,
    /// Scheduled start (MJD) of observation
    pub scheduled_start_mjd: f64,
    /// Scheduled end (MJD) of observation
    pub scheduled_end_mjd: f64,
    /// Scheduled duration of observation
    pub scheduled_duration_milliseconds: u64,
    /// RA tile pointing
    pub ra_tile_pointing_degrees: f64,
    /// DEC tile pointing
    pub dec_tile_pointing_degrees: f64,
    /// RA phase centre
    pub ra_phase_center_degrees: Option<f64>,
    /// DEC phase centre
    pub dec_phase_center_degrees: Option<f64>,
    /// AZIMUTH
    pub azimuth_degrees: f64,
    /// ALTITUDE
    pub altitude_degrees: f64,
    /// Altitude of Sun
    pub sun_altitude_degrees: f64,
    /// Distance from pointing center to Sun
    pub sun_distance_degrees: f64,
    /// Distance from pointing center to the Moon
    pub moon_distance_degrees: f64,
    /// Distance from pointing center to Jupiter
    pub jupiter_distance_degrees: f64,
    /// Local Sidereal Time
    pub lst_degrees: f64,
    /// Hour Angle of pointing center (as a string)
    pub hour_angle_string: String,
    /// GRIDNAME
    pub grid_name: String,
    /// GRIDNUM
    pub grid_number: i32,
    /// CREATOR
    pub creator: String,
    /// PROJECT
    pub project_id: String,
    /// Observation name
    pub observation_name: String,
    /// MWA observation mode
    pub mode: String,
    /// RECVRS    // Array of receiver numbers (this tells us how many receivers too)
    pub receivers: Vec<usize>,
    /// DELAYS    // Array of delays
    pub delays: Vec<usize>,
    /// ATTEN_DB  // global analogue attenuation, in dB
    pub global_analogue_attenuation_db: f64,
    /// Seconds of bad data after observation starts
    pub quack_time_duration_milliseconds: u64,
    /// OBSID+QUACKTIM as Unix timestamp (first good timestep)
    pub good_time_unix_milliseconds: u64,
    /// Version of the correlator format
    pub corr_version: CorrelatorVersion,
    /// The proper start of the observation (the time that is common to all
    /// provided gpubox files).
    pub start_unix_time_milliseconds: u64,
    /// `end_time_milliseconds` will is the actual end time of the observation
    /// i.e. start time of last common timestep plus integration time.
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
    /// Baslines
    pub baselines: Vec<mwalibBaseline>,
    /// Total number of rf_inputs (tiles * 2 pols X&Y)
    pub num_rf_inputs: usize,
    /// The Metafits defines an rf chain for antennas(tiles) * pol(X,Y)
    pub rf_inputs: Vec<mwalibRFInput>,
    /// Number of antenna pols. e.g. X and Y
    pub num_antenna_pols: usize,
    /// Number of polarisation combinations in the visibilities e.g. XX,XY,YX,YY == 4
    pub num_visibility_pols: usize,
    /// Visibility polarisations
    pub visibility_pols: Vec<mwalibVisibilityPol>,
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
    pub fn new<T: AsRef<std::path::Path>>(
        metafits: &T,
        gpuboxes: &[T],
    ) -> Result<Self, MwalibError> {
        // Pull out observation details. Save the metafits HDU for faster
        // accesses.
        let mut metafits_fptr = fits_open!(&metafits)?;
        let metafits_hdu = fits_open_hdu!(&mut metafits_fptr, 0)?;
        let metafits_tile_table_hdu = fits_open_hdu!(&mut metafits_fptr, 1)?;

        // Populate obsid from the metafits
        let obsid = get_required_fits_key!(&mut metafits_fptr, &metafits_hdu, "GPSTIME")?;

        // from MWA_Tools/CONV2UVFITS/convutils.h
        // Used to determine electrical lengths if EL_ not present in metafits for an rf_input
        let coax_v_factor: f64 = 1.204;

        let mwa_latitude_radians: f64 = dms_to_degrees(-26, 42, 11.94986).to_radians(); // -26d42m11.94986s
        let mwa_longitude_radians: f64 = dms_to_degrees(116, 40, 14.93485).to_radians(); // 116d40m14.93485s
        let mwa_altitude_metres: f64 = 377.827;

        let integration_time_milliseconds: u64 = {
            let it: f64 = get_required_fits_key!(&mut metafits_fptr, &metafits_hdu, "INTTIME")?;
            (it * 1000.) as _
        };
        let quack_time_duration_milliseconds: u64 = {
            let qt: f64 = get_required_fits_key!(&mut metafits_fptr, &metafits_hdu, "QUACKTIM")?;
            (qt * 1000.).round() as _
        };
        let good_time_unix_milliseconds: u64 = {
            let gt: f64 = get_required_fits_key!(&mut metafits_fptr, &metafits_hdu, "GOODTIME")?;
            (gt * 1000.).round() as _
        };

        let (mut gpubox_info, timesteps) =
        // Do gpubox stuff only if we have gpubox files.
            if !gpuboxes.is_empty() {
                let gpubox_info = examine_gpubox_files(&gpuboxes)?;
                // We can unwrap here because the `gpubox_time_map` can't be empty if
                // `gpuboxes` isn't empty.
                let timesteps = mwalibTimeStep::populate_timesteps(&gpubox_info.time_map).unwrap();
                (gpubox_info, timesteps)
            } else {
                // If there are no gpubox files, then we need to use metafits info.
                let nscans: u64 = get_required_fits_key!(&mut metafits_fptr, &metafits_hdu, "NSCANS")?;
                let timesteps: Vec<mwalibTimeStep> = (0..nscans)
                    .map(|i| {
                        let time = good_time_unix_milliseconds + i * integration_time_milliseconds;
                        mwalibTimeStep::new(time)
                    })
                    .collect();

                // Make a fake `gpubox_time_map`.
                let channels: Vec<usize> = {
                    let s: String = get_required_fits_key!(&mut metafits_fptr, &metafits_hdu, "CHANSEL")?;
                    s.replace(&['\'', '&'][..], "")
                        .split(',')
                        .map(|s| s.parse::<usize>().unwrap() + 1)
                        .collect()
                };

                let mut gpubox_time_map = BTreeMap::new();
                for (i, time) in timesteps.iter().enumerate() {
                    for channel in &channels {
                        gpubox_time_map
                            .entry(time.unix_time_ms)
                            .or_insert_with(BTreeMap::new)
                            .entry(*channel)
                            .or_insert((0, i));
                    }
                }

                (GpuboxInfo {
                    batches: vec![],
                    corr_format: CorrelatorVersion::Legacy,
                    time_map: gpubox_time_map,
                    hdu_size: 0
                }, timesteps)
            };
        let num_timesteps = timesteps.len();

        // Create a vector of rf_input structs from the metafits
        let num_rf_inputs: usize =
            get_required_fits_key!(&mut metafits_fptr, &metafits_hdu, "NINPUTS")?;

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

        // Populate baselines
        let baselines = mwalibBaseline::populate_baselines(num_antennas);

        // Always assume that MWA antennas have 2 pols
        let num_antenna_pols = 2;

        // Populate the pols that come out of the correlator
        let visibility_pols = mwalibVisibilityPol::populate_visibility_pols();
        let num_visibility_pols = visibility_pols.len();

        // `num_baselines` is the number of cross-correlations + the number of
        // auto-correlations.
        let num_baselines = (num_antennas / 2) * (num_antennas + 1);

        // observation bandwidth (read from metafits in MHz)
        let metafits_observation_bandwidth_hz: u32 = {
            let bw: f64 = get_required_fits_key!(&mut metafits_fptr, &metafits_hdu, "BANDWDTH")?;
            (bw * 1e6).round() as _
        };

        // Populate coarse channels
        let (coarse_channels, num_coarse_channels, coarse_channel_width_hz) =
            coarse_channel::mwalibCoarseChannel::populate_coarse_channels(
                &mut metafits_fptr,
                &metafits_hdu,
                gpubox_info.corr_format,
                metafits_observation_bandwidth_hz,
                &gpubox_info.time_map,
            )?;
        let observation_bandwidth_hz = (num_coarse_channels as u32) * coarse_channel_width_hz;

        // Fine-channel resolution. The FINECHAN value in the metafits is in units
        // of kHz - make it Hz.
        let fine_channel_width_hz: u32 = {
            let fc: f64 = get_required_fits_key!(&mut metafits_fptr, &metafits_hdu, "FINECHAN")?;
            (fc * 1000.).round() as _
        };

        // Determine the number of fine channels per coarse channel.
        let num_fine_channels_per_coarse =
            (coarse_channel_width_hz / fine_channel_width_hz) as usize;

        // We have enough information to validate HDU matches metafits
        if !gpuboxes.is_empty() {
            let coarse_channel = coarse_channels[0].gpubox_number;
            let (batch_index, _) =
                gpubox_info.time_map[&timesteps[0].unix_time_ms][&coarse_channel];

            let mut fptr = &mut gpubox_info.batches[batch_index].gpubox_files[0].fptr;

            mwalibContext::validate_first_hdu(
                gpubox_info.corr_format,
                num_fine_channels_per_coarse,
                num_baselines,
                num_visibility_pols,
                &mut fptr,
            )?;
        }

        // Populate the start and end times of the observation.
        // Start= start of first timestep
        // End  = start of last timestep + integration time
        let (start_unix_time_milliseconds, end_unix_time_milliseconds, duration_milliseconds) = {
            let o = determine_obs_times(&gpubox_info.time_map, integration_time_milliseconds)?;
            (o.start_millisec, o.end_millisec, o.duration_millisec)
        };

        // populate lots of useful metadata
        let scheduled_start_utc_string: String =
            get_required_fits_key!(&mut metafits_fptr, &metafits_hdu, "DATE-OBS")?;

        let scheduled_start_utc_string_with_offset: String = scheduled_start_utc_string + "+00:00";

        let scheduled_start_utc =
            DateTime::parse_from_rfc3339(&scheduled_start_utc_string_with_offset)
                .expect("Unable to parse DATE-OBS into a date time");
        let scheduled_start_mjd: f64 =
            get_required_fits_key!(&mut metafits_fptr, &metafits_hdu, "MJD")?;
        let scheduled_duration_milliseconds: u64 = {
            let ex: u64 = get_required_fits_key!(&mut metafits_fptr, &metafits_hdu, "EXPOSURE")?;
            ex * 1000
        };
        let scheduled_end_utc =
            scheduled_start_utc + Duration::milliseconds(scheduled_duration_milliseconds as i64);

        // To increment the mjd we need to fractional proportion of the day that the duration represents
        let scheduled_end_mjd =
            scheduled_start_mjd + (scheduled_duration_milliseconds as f64 / 1000. / 86400.);

        let scheduled_start_gpstime_milliseconds: u64 = obsid as u64 * 1000;
        let scheduled_end_gpstime_milliseconds: u64 =
            scheduled_start_gpstime_milliseconds + scheduled_duration_milliseconds;

        let scheduled_start_unix_time_milliseconds: u64 =
            good_time_unix_milliseconds - quack_time_duration_milliseconds;
        let scheduled_end_unix_time_milliseconds: u64 =
            scheduled_start_unix_time_milliseconds + scheduled_duration_milliseconds;

        let ra_tile_pointing_degrees: f64 =
            get_required_fits_key!(&mut metafits_fptr, &metafits_hdu, "RA")?;
        let dec_tile_pointing_degrees: f64 =
            get_required_fits_key!(&mut metafits_fptr, &metafits_hdu, "DEC")?;
        let ra_phase_center_degrees: Option<f64> =
            get_optional_fits_key!(&mut metafits_fptr, &metafits_hdu, "RAPHASE")?;
        let dec_phase_center_degrees: Option<f64> =
            get_optional_fits_key!(&mut metafits_fptr, &metafits_hdu, "DECPHASE")?;
        let azimuth_degrees: f64 =
            get_required_fits_key!(&mut metafits_fptr, &metafits_hdu, "AZIMUTH")?;
        let altitude_degrees: f64 =
            get_required_fits_key!(&mut metafits_fptr, &metafits_hdu, "ALTITUDE")?;
        let sun_altitude_degrees: f64 =
            get_required_fits_key!(&mut metafits_fptr, &metafits_hdu, "SUN-ALT")?;
        let sun_distance_degrees: f64 =
            get_required_fits_key!(&mut metafits_fptr, &metafits_hdu, "SUN-DIST")?;
        let moon_distance_degrees: f64 =
            get_required_fits_key!(&mut metafits_fptr, &metafits_hdu, "MOONDIST")?;
        let jupiter_distance_degrees: f64 =
            get_required_fits_key!(&mut metafits_fptr, &metafits_hdu, "JUP-DIST")?;
        let lst_degrees: f64 = get_required_fits_key!(&mut metafits_fptr, &metafits_hdu, "LST")?;
        let hour_angle_string = get_required_fits_key!(&mut metafits_fptr, &metafits_hdu, "HA")?;
        let grid_name = get_required_fits_key!(&mut metafits_fptr, &metafits_hdu, "GRIDNAME")?;
        let grid_number = get_required_fits_key!(&mut metafits_fptr, &metafits_hdu, "GRIDNUM")?;
        let creator = get_required_fits_key!(&mut metafits_fptr, &metafits_hdu, "CREATOR")?;
        let project_id = get_required_fits_key!(&mut metafits_fptr, &metafits_hdu, "PROJECT")?;
        let observation_name =
            get_required_fits_key!(&mut metafits_fptr, &metafits_hdu, "FILENAME")?;
        let mode = get_required_fits_key!(&mut metafits_fptr, &metafits_hdu, "MODE")?;
        let receivers_string: String =
            get_required_fits_key!(&mut metafits_fptr, &metafits_hdu, "RECVRS")?;

        let receivers: Vec<usize> = receivers_string
            .replace(&['\'', '&'][..], "")
            .split(',')
            .map(|s| s.parse().unwrap())
            .collect();

        let delays_string: String =
            get_required_fits_key!(&mut metafits_fptr, &metafits_hdu, "DELAYS")?;

        let delays: Vec<usize> = delays_string
            .replace(&['\'', '&'][..], "")
            .split(',')
            .map(|s| s.parse().unwrap())
            .collect();

        let global_analogue_attenuation_db: f64 =
            get_required_fits_key!(&mut metafits_fptr, &metafits_hdu, "ATTEN_DB")?;
        // Prepare the conversion array to convert legacy correlator format into mwax format
        // or just leave it empty if we're in any other format
        let legacy_conversion_table: Vec<mwalibLegacyConversionBaseline> =
            match gpubox_info.corr_format {
                CorrelatorVersion::OldLegacy | CorrelatorVersion::Legacy => {
                    convert::generate_conversion_array(&mut rf_inputs)
                }
                _ => Vec::new(),
            };

        // Sort the rf_inputs back into the correct output order
        rf_inputs.sort_by_key(|k| k.subfile_order);

        Ok(mwalibContext {
            coax_v_factor,
            mwa_latitude_radians,
            mwa_longitude_radians,
            mwa_altitude_metres,
            corr_version: gpubox_info.corr_format,
            obsid,
            scheduled_start_unix_time_milliseconds,
            scheduled_end_unix_time_milliseconds,
            scheduled_start_gpstime_milliseconds,
            scheduled_end_gpstime_milliseconds,
            scheduled_start_utc,
            scheduled_end_utc,
            scheduled_start_mjd,
            scheduled_end_mjd,
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
            receivers,
            delays,
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
            baselines,
            num_rf_inputs,
            rf_inputs,
            integration_time_milliseconds,
            num_antenna_pols,
            num_visibility_pols,
            visibility_pols,
            num_fine_channels_per_coarse,
            num_coarse_channels,
            coarse_channels,
            coarse_channel_width_hz,
            fine_channel_width_hz,
            observation_bandwidth_hz,
            metafits_filename: metafits
                .as_ref()
                .to_str()
                .expect("Metafits filename is not UTF-8 compliant")
                .to_string(),
            gpubox_batches: gpubox_info.batches,
            gpubox_time_map: gpubox_info.time_map,
            num_gpubox_files: gpuboxes.len(),
            num_timestep_coarse_channel_bytes: gpubox_info.hdu_size * 4,
            num_timestep_coarse_channel_floats: gpubox_info.hdu_size,
            legacy_conversion_table,
        })
    }

    /// Validates the first HDU of a gpubox file against metafits metadata
    ///
    /// In this case we call `validate_hdu_axes()`
    ///
    /// # Arguments
    ///
    /// * `corr_version` - Correlator version of this gpubox file.
    ///
    /// * `metafits_fine_channels_per_coarse` - the number of fine chan per coarse as calculated using info from metafits.
    ///
    /// * `metafits_baselines` - the number of baselines as reported by the metafits file.
    ///
    /// * `visibility_pols` - the number of pols produced by the correlator (always 4 for MWA)
    ///
    /// * `gpubox_fptr` - FITSFile pointer to an MWA GPUbox file
    ///
    /// # Returns
    ///
    /// * Result containing `Ok` if it is valid, or a custom `MwalibError` if not valid.
    ///
    ///
    pub fn validate_first_hdu(
        corr_version: CorrelatorVersion,
        metafits_fine_channels_per_coarse: usize,
        metafits_baselines: usize,
        visibility_pols: usize,
        gpubox_fptr: &mut fitsio::FitsFile,
    ) -> Result<(), GpuboxError> {
        // Get NAXIS1 and NAXIS2 from a gpubox file first image HDU
        let hdu = fits_open_hdu!(gpubox_fptr, 1)?;
        let dimensions = get_hdu_image_size!(gpubox_fptr, &hdu)?;
        let naxis1 = dimensions[1];
        let naxis2 = dimensions[0];

        Self::validate_hdu_axes(
            corr_version,
            metafits_fine_channels_per_coarse,
            metafits_baselines,
            visibility_pols,
            naxis1,
            naxis2,
        )
    }

    /// Validates the first HDU of a gpubox file against metafits metadata
    ///
    /// In this case we check that NAXIS1 = the correct value and NAXIS2 = the correct value calculated from the metafits
    ///
    /// # Arguments
    ///
    /// * `corr_version` - Correlator version of this gpubox file.
    ///
    /// * `metafits_fine_channels_per_coarse` - the number of fine chan per coarse as calculated using info from metafits.
    ///
    /// * `metafits_baselines` - the number of baselines as reported by the metafits file.
    ///
    /// * `visibility_pols` - the number of pols produced by the correlator (always 4 for MWA)
    ///
    /// * `naxis1` - NAXIS1 keyword read from HDU1 of a Gpubox file
    ///
    /// * `naxis2` - NAXIS2 keyword read from HDU1 of a Gpubox file
    ///
    /// # Returns
    ///
    /// * Result containing `Ok` if it is valid, or a custom `MwalibError` if not valid.
    ///
    ///
    pub fn validate_hdu_axes(
        corr_version: CorrelatorVersion,
        metafits_fine_channels_per_coarse: usize,
        metafits_baselines: usize,
        visibility_pols: usize,
        naxis1: usize,
        naxis2: usize,
    ) -> Result<(), GpuboxError> {
        // We have different values depending on the version of the correlator
        match corr_version {
            CorrelatorVersion::OldLegacy | CorrelatorVersion::Legacy => {
                // NAXIS1 = baselines * visibility_pols * 2
                // NAXIS2 = fine channels
                let calculated_naxis1: i32 = metafits_baselines as i32 * visibility_pols as i32 * 2;
                let calculated_naxis2: i32 = metafits_fine_channels_per_coarse as i32;

                if calculated_naxis1 != naxis1 as i32 {
                    return Err(GpuboxError::LegacyNAXIS1Mismatch {
                        naxis1,
                        calculated_naxis1,
                        metafits_baselines,
                        visibility_pols,
                        naxis2,
                    });
                }
                if calculated_naxis2 != naxis2 as i32 {
                    return Err(GpuboxError::LegacyNAXIS2Mismatch {
                        naxis2,
                        calculated_naxis2,
                        metafits_fine_channels_per_coarse,
                    });
                }
            }
            CorrelatorVersion::V2 => {
                // NAXIS1 = fine channels * visibility pols * 2
                // NAXIS2 = baselines
                let calculated_naxis1: i32 =
                    metafits_fine_channels_per_coarse as i32 * visibility_pols as i32 * 2;
                let calculated_naxis2: i32 = metafits_baselines as i32;

                if calculated_naxis1 != naxis1 as i32 {
                    return Err(GpuboxError::MwaxNAXIS1Mismatch {
                        naxis1,
                        calculated_naxis1,
                        metafits_fine_channels_per_coarse,
                        visibility_pols,
                        naxis2,
                    });
                }
                if calculated_naxis2 != naxis2 as i32 {
                    return Err(GpuboxError::MwaxNAXIS2Mismatch {
                        naxis2,
                        calculated_naxis2,
                        metafits_baselines,
                    });
                }
            }
        }

        Ok(())
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
    ) -> Result<Vec<f32>, GpuboxError> {
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

        if self.gpubox_batches.is_empty() {
            return Err(GpuboxError::NoGpuboxes);
        }
        let mut fptr =
            &mut self.gpubox_batches[batch_index].gpubox_files[coarse_channel_index].fptr;
        let hdu = fits_open_hdu!(&mut fptr, hdu_index)?;
        output_buffer = get_fits_image!(&mut fptr, &hdu)?;
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
    ) -> Result<Vec<f32>, GpuboxError> {
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

        if self.gpubox_batches.is_empty() {
            return Err(GpuboxError::NoGpuboxes);
        }
        let mut fptr =
            &mut self.gpubox_batches[batch_index].gpubox_files[coarse_channel_index].fptr;
        let hdu = fits_open_hdu!(&mut fptr, hdu_index)?;
        output_buffer = get_fits_image!(&mut fptr, &hdu)?;
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
#[cfg(not(tarpaulin_include))]
impl fmt::Display for mwalibContext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // `size` is the number of floats (self.gpubox_hdu_size) multiplied by 4
        // bytes per float, divided by 1024^2 to get MiB.
        let size = (self.num_timestep_coarse_channel_floats * 4) as f64 / (1024 * 1024) as f64;
        writeln!(
            f,
            r#"mwalibContext (
    Correlator version:       {corr_ver},

    MWA latitude:             {mwa_lat} degrees,
    MWA longitude:            {mwa_lon} degrees
    MWA altitude:             {mwa_alt} m,

    obsid:                    {obsid},

    Creator:                  {creator},
    Project ID:               {project_id},
    Observation Name:         {obs_name},
    Receivers:                {receivers:?},
    Delays:                   {delays:?},
    Global attenuation:       {atten} dB,

    Scheduled start (UNIX)    {sched_start_unix},
    Scheduled end (UNIX)      {sched_end_unix},
    Scheduled start (GPS)     {sched_start_gps},
    Scheduled end (GPS)       {sched_end_gps},
    Scheduled start (utc)     {sched_start_utc},
    Scheduled end (utc)       {sched_end_utc},
    Scheduled start (MJD)     {sched_start_mjd},
    Scheduled end (MJD)       {sched_end_mjd},
    Scheduled duration        {sched_duration} s,
    Actual UNIX start time:   {start_unix},
    Actual UNIX end time:     {end_unix},
    Actual duration:          {duration} s,
    Quack time:               {quack_duration} s,
    Good UNIX start time:     {good_time},

    R.A. (tile_pointing):     {rtpc} degrees,
    Dec. (tile_pointing):     {dtpc} degrees,
    R.A. (phase center):      {rppc},
    Dec. (phase center):      {dppc},
    Azimuth:                  {az} degrees,
    Altitude:                 {alt} degrees,
    Sun altitude:             {sun_alt} degrees,
    Sun distance:             {sun_dis} degrees,
    Moon distance:            {moon_dis} degrees,
    Jupiter distance:         {jup_dis} degrees,
    LST:                      {lst} degrees,
    Hour angle:               {ha} degrees,
    Grid name:                {grid},
    Grid number:              {grid_n},

    num timesteps:            {n_timesteps},
    timesteps:                {timesteps:?},

    num antennas:             {n_ants},
    antennas:                 {ants:?},
    rf_inputs:                {rfs:?},

    num baselines:            {n_bls},
    baselines:                {bl01} v {bl02} to {bll1} v {bll2}
    num auto-correlations:    {n_ants},
    num cross-correlations:   {n_ccs},

    num antenna pols:         {n_aps},
    num visibility pols:      {n_vps},
    visibility pols:          {vp0}, {vp1}, {vp2}, {vp3},

    observation bandwidth:    {obw} MHz,
    num coarse channels,      {n_coarse},
    coarse channels:          {coarse:?},

    Correlator Mode:
    Mode:                     {mode},
    fine channel resolution:  {fcw} kHz,
    integration time:         {int_time:.2} s
    num fine channels/coarse: {nfcpc},

    gpubox HDU size:          {hdu_size} MiB,
    Memory usage per scan:    {scan_size} MiB,

    metafits filename:        {meta},
    gpubox batches:           {batches:#?},
)"#,
            corr_ver = self.corr_version,
            mwa_lat = self.mwa_latitude_radians.to_degrees(),
            mwa_lon = self.mwa_longitude_radians.to_degrees(),
            mwa_alt = self.mwa_altitude_metres,
            obsid = self.obsid,
            creator = self.creator,
            project_id = self.project_id,
            obs_name = self.observation_name,
            receivers = self.receivers,
            delays = self.delays,
            atten = self.global_analogue_attenuation_db,
            sched_start_unix = self.scheduled_start_unix_time_milliseconds as f64 / 1e3,
            sched_end_unix = self.scheduled_end_unix_time_milliseconds as f64 / 1e3,
            sched_start_gps = self.scheduled_start_gpstime_milliseconds as f64 / 1e3,
            sched_end_gps = self.scheduled_end_gpstime_milliseconds as f64 / 1e3,
            sched_start_utc = self.scheduled_start_utc,
            sched_end_utc = self.scheduled_end_utc,
            sched_start_mjd = self.scheduled_start_mjd,
            sched_end_mjd = self.scheduled_end_mjd,
            sched_duration = self.scheduled_duration_milliseconds as f64 / 1e3,
            start_unix = self.start_unix_time_milliseconds as f64 / 1e3,
            end_unix = self.end_unix_time_milliseconds as f64 / 1e3,
            duration = self.duration_milliseconds as f64 / 1e3,
            quack_duration = self.quack_time_duration_milliseconds as f64 / 1e3,
            good_time = self.good_time_unix_milliseconds as f64 / 1e3,
            rtpc = self.ra_tile_pointing_degrees,
            dtpc = self.dec_tile_pointing_degrees,
            rppc = if let Some(rppc) = self.ra_phase_center_degrees {
                format!("{} degrees", rppc)
            } else {
                "N/A".to_string()
            },
            dppc = if let Some(dppc) = self.dec_phase_center_degrees {
                format!("{} degrees", dppc)
            } else {
                "N/A".to_string()
            },
            az = self.azimuth_degrees,
            alt = self.altitude_degrees,
            sun_alt = self.sun_altitude_degrees,
            sun_dis = self.sun_distance_degrees,
            moon_dis = self.moon_distance_degrees,
            jup_dis = self.jupiter_distance_degrees,
            lst = self.lst_degrees,
            ha = self.hour_angle_string,
            grid = self.grid_name,
            grid_n = self.grid_number,
            n_timesteps = self.num_timesteps,
            timesteps = self.timesteps,
            n_ants = self.num_antennas,
            ants = self.antennas,
            rfs = self.rf_inputs,
            n_bls = self.num_baselines,
            bl01 = self.baselines[0].antenna1_index,
            bl02 = self.baselines[0].antenna2_index,
            bll1 = self.baselines[self.num_baselines - 1].antenna1_index,
            bll2 = self.baselines[self.num_baselines - 1].antenna2_index,
            n_ccs = self.num_baselines - self.num_antennas,
            n_aps = self.num_antenna_pols,
            n_vps = self.num_visibility_pols,
            vp0 = self.visibility_pols[0].polarisation,
            vp1 = self.visibility_pols[1].polarisation,
            vp2 = self.visibility_pols[2].polarisation,
            vp3 = self.visibility_pols[3].polarisation,
            obw = self.observation_bandwidth_hz as f64 / 1e6,
            n_coarse = self.num_coarse_channels,
            coarse = self.coarse_channels,
            mode = self.mode,
            fcw = self.fine_channel_width_hz as f64 / 1e3,
            int_time = self.integration_time_milliseconds as f64 / 1e3,
            nfcpc = self.num_fine_channels_per_coarse,
            hdu_size = size,
            scan_size = size * self.num_gpubox_files as f64,
            meta = self.metafits_filename,
            batches = self.gpubox_batches,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use float_cmp::*;

    #[test]
    fn test_context_new_missing_gpubox_files() {
        let metafits_filename = "test_files/1101503312_1_timestep/1101503312.metafits";
        let gpuboxfiles = Vec::new();

        // No gpubox files provided
        let context = mwalibContext::new(&metafits_filename, &gpuboxfiles);
        assert!(context.is_ok());
        let context = context.unwrap();

        assert!(context.gpubox_batches.is_empty());
    }

    #[test]
    fn test_context_new_invalid_metafits() {
        let metafits_filename = "invalid.metafits";
        let filename =
            "test_files/1101503312_1_timestep/1101503312_20141201210818_gpubox01_00.fits";
        let gpuboxfiles = vec![filename];

        // No gpubox files provided
        let context = mwalibContext::new(&metafits_filename, &gpuboxfiles);

        assert!(context.is_err());
    }

    #[test]
    #[allow(clippy::cognitive_complexity)]
    fn test_context_legacy_v1() {
        // Open the test mwax file
        let metafits_filename = "test_files/1101503312_1_timestep/1101503312.metafits";
        let filename =
            "test_files/1101503312_1_timestep/1101503312_20141201210818_gpubox01_00.fits";

        //
        // Read the observation using mwalib
        //
        // Open a context and load in a test metafits and gpubox file
        let gpuboxfiles = vec![filename];
        let context = mwalibContext::new(&metafits_filename, &gpuboxfiles)
            .expect("Failed to create mwalibContext");

        // Test the properties of the context object match what we expect
        // Correlator version:       v1 Legacy,
        assert_eq!(context.corr_version, CorrelatorVersion::Legacy);

        // MWA latitude:             -26.703319405555554 degrees,
        assert!(approx_eq!(
            f64,
            context.mwa_latitude_radians.to_degrees(),
            -26.703_319_405_555_554,
            F64Margin::default()
        ));
        // MWA longitude:            116.67081523611111 degrees
        assert!(approx_eq!(
            f64,
            context.mwa_longitude_radians.to_degrees(),
            116.670_815_236_111_11,
            F64Margin::default()
        ));
        // MWA altitude:             377.827 m,
        assert!(approx_eq!(
            f64,
            context.mwa_altitude_metres,
            377.827,
            F64Margin::default()
        ));

        // obsid:                    1101503312,
        assert_eq!(context.obsid, 1_101_503_312);

        // Creator:                  Randall,
        assert_eq!(context.creator, "Randall");

        // Project ID:               G0009,
        assert_eq!(context.project_id, "G0009");

        // Observation Name:         FDS_DEC-26.7_121,
        assert_eq!(context.observation_name, "FDS_DEC-26.7_121");

        // Receivers:                [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
        assert_eq!(context.receivers.len(), 16);
        assert_eq!(context.receivers[0], 1);
        assert_eq!(context.receivers[15], 16);

        // Delays:                   [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        assert_eq!(context.delays.len(), 16);
        assert_eq!(context.delays[0], 0);
        assert_eq!(context.delays[15], 0);

        // Global attenuation:       1 dB,
        assert_eq!(context.global_analogue_attenuation_db as i16, 1);

        // Scheduled start (utc)     2014-12-01 21:08:16 +00:00,
        assert_eq!(
            context.scheduled_start_utc,
            DateTime::parse_from_rfc3339("2014-12-01T21:08:16+00:00").unwrap()
        );

        // Scheduled start (MJD)     56992.88074074074,
        assert!(approx_eq!(
            f64,
            context.scheduled_start_mjd,
            56_992.880_740_740_74,
            F64Margin::default()
        ));

        // Scheduled duration        112 s,
        assert_eq!(context.scheduled_duration_milliseconds, 112_000);

        // Actual UNIX start time:   1417468096,
        assert_eq!(context.start_unix_time_milliseconds, 1_417_468_096_000);

        // Actual UNIX end time:     1417468098,
        assert_eq!(context.end_unix_time_milliseconds, 1_417_468_098_000);

        // Actual duration:          2 s,
        assert_eq!(context.duration_milliseconds, 2000);

        // Quack time:               2 s,
        assert_eq!(context.quack_time_duration_milliseconds, 2000);

        // Good UNIX start time:     1417468098,
        assert_eq!(context.good_time_unix_milliseconds, 1_417_468_098_000);

        // R.A. (tile_pointing):     144.2107504850443 degrees,
        assert!(approx_eq!(
            f64,
            context.ra_tile_pointing_degrees,
            144.210_750_485_044_3,
            F64Margin::default()
        ));

        // Dec. (tile_pointing):     -26.63403125476213 degrees,
        assert!(approx_eq!(
            f64,
            context.dec_tile_pointing_degrees,
            -26.634_031_254_762_13,
            F64Margin::default()
        ));

        // R.A. (phase center):      None degrees,
        assert!(context.ra_phase_center_degrees.is_none());

        // Dec. (phase center):      None degrees,
        assert!(context.dec_phase_center_degrees.is_none());

        // Azimuth:                  0 degrees,
        assert!(approx_eq!(
            f64,
            context.azimuth_degrees,
            0.,
            F64Margin::default()
        ));

        // Altitude:                 90 degrees,
        assert!(approx_eq!(
            f64,
            context.altitude_degrees,
            90.,
            F64Margin::default()
        ));

        // Sun altitude:             -1.53222775573148 degrees,
        assert!(approx_eq!(
            f64,
            context.sun_altitude_degrees,
            -1.532_227_755_731_48,
            F64Margin::default()
        ));

        // Sun distance:             91.5322277557315 degrees,
        assert!(approx_eq!(
            f64,
            context.sun_distance_degrees,
            91.532_227_755_731_5,
            F64Margin::default()
        ));

        // Moon distance:            131.880015235607 degrees,
        assert!(approx_eq!(
            f64,
            context.moon_distance_degrees,
            131.880_015_235_607,
            F64Margin::default()
        ));

        // Jupiter distance:         41.401684338269 degrees,
        assert!(approx_eq!(
            f64,
            context.jupiter_distance_degrees,
            41.401_684_338_269,
            F64Margin::default()
        ));

        // LST:                      144.381251875516 degrees,
        assert!(approx_eq!(
            f64,
            context.lst_degrees,
            144.381_251_875_516,
            F64Margin::default()
        ));

        // Hour angle:               -00:00:00.00 degrees,
        // Grid name:                sweet,
        assert_eq!(context.grid_name, "sweet");

        // Grid number:              0,
        assert_eq!(context.grid_number, 0);

        // num timesteps:            1,
        assert_eq!(context.num_timesteps, 1);

        // timesteps:                [unix=1417468096.000],
        assert_eq!(context.timesteps[0].unix_time_ms, 1_417_468_096_000);

        // num antennas:             128,
        assert_eq!(context.num_antennas, 128);

        // antennas:                 [Tile011, Tile012, ... Tile167, Tile168],
        assert_eq!(context.antennas[0].tile_name, "Tile011");
        assert_eq!(context.antennas[127].tile_name, "Tile168");

        // rf_inputs:                [Tile011X, Tile011Y, ... Tile168X, Tile168Y],
        assert_eq!(context.num_rf_inputs, 256);
        assert_eq!(context.rf_inputs[0].pol, Pol::X);
        assert_eq!(context.rf_inputs[0].tile_name, "Tile011");
        assert_eq!(context.rf_inputs[255].pol, Pol::Y);
        assert_eq!(context.rf_inputs[255].tile_name, "Tile168");

        // num baselines:            8256,
        assert_eq!(context.num_baselines, 8256);

        // num antenna pols:         2,
        assert_eq!(context.num_antenna_pols, 2);

        // num visibility pols:      4,
        assert_eq!(context.num_visibility_pols, 4);

        // observation bandwidth:    1.28 MHz,
        assert_eq!(context.observation_bandwidth_hz, 1_280_000);

        // num coarse channels,      1,
        assert_eq!(context.num_coarse_channels, 1);

        // coarse channels:          [gpu=1 corr=0 rec=109 @ 139.520 MHz],
        assert_eq!(context.coarse_channels[0].gpubox_number, 1);
        assert_eq!(context.coarse_channels[0].receiver_channel_number, 109);
        assert_eq!(context.coarse_channels[0].channel_centre_hz, 139_520_000);

        // Correlator Mode:
        // Mode:                     HW_LFILES,
        assert_eq!(context.mode, "HW_LFILES");

        // fine channel resolution:  10 kHz,
        assert_eq!(context.fine_channel_width_hz, 10_000);

        // integration time:         2.00 s
        assert_eq!(context.integration_time_milliseconds, 2000);

        // num fine channels/coarse: 128,
        assert_eq!(context.num_fine_channels_per_coarse, 128);

        // gpubox HDU size:          32.25 MiB,
        // Memory usage per scan:    32.25 MiB,

        // metafits filename:        ../test_files/1101503312_1_timestep/1101503312.metafits,
        // gpubox batches:           [
        // batch_number=0 gpubox_files=[filename=../test_files/1101503312_1_timestep/1101503312_20141201210818_gpubox01_00.fits channelidentifier=1]
    }

    #[test]
    fn test_mwax_read() {
        // Open the test mwax file
        // a) directly using Fits  (data will be ordered [baseline][freq][pol][r][i])
        // b) using mwalib (by baseline) (data will be ordered the same as the raw fits file)
        // c) using mwalib (by frequency) (data will be ordered [freq][baseline][pol][r][i])
        // Then check b) is the same as a) and
        // that c) is the same size and sum as a) and b)
        let mwax_metafits_filename = "test_files/1244973688_1_timestep/1244973688.metafits";
        let mwax_filename =
            "test_files/1244973688_1_timestep/1244973688_20190619100110_ch114_000.fits";

        //
        // Read the mwax file using FITS
        //
        let mut fptr = fits_open!(&mwax_filename).unwrap();
        let fits_hdu = fits_open_hdu!(&mut fptr, 1).unwrap();

        // Read data from fits hdu into vector
        let fits_hdu_data: Vec<f32> = get_fits_image!(&mut fptr, &fits_hdu).unwrap();
        //
        // Read the mwax file by frequency using mwalib
        //
        // Open a context and load in a test metafits and gpubox file
        let gpuboxfiles = vec![mwax_filename];
        let mut context = mwalibContext::new(&mwax_metafits_filename, &gpuboxfiles)
            .expect("Failed to create mwalibContext");

        // Read and convert first HDU by baseline
        let mwalib_hdu_data_by_bl: Vec<f32> = context.read_by_baseline(0, 0).expect("Error!");

        // Read and convert first HDU by frequency
        let mwalib_hdu_data_by_freq: Vec<f32> = context.read_by_frequency(0, 0).expect("Error!");

        // First assert that the data vectors are the same size
        assert_eq!(fits_hdu_data.len(), mwalib_hdu_data_by_bl.len());
        assert_eq!(fits_hdu_data.len(), mwalib_hdu_data_by_freq.len());

        // Check all 3 sum to the same value
        let sum_fits: f64 = fits_hdu_data.iter().fold(0., |sum, x| sum + *x as f64);
        let sum_freq: f64 = mwalib_hdu_data_by_freq
            .iter()
            .fold(0., |sum, x| sum + *x as f64);
        let sum_bl: f64 = mwalib_hdu_data_by_bl
            .iter()
            .fold(0., |sum, x| sum + *x as f64);

        // Check sums match
        assert_eq!(
            approx_eq!(f64, sum_fits, sum_freq, F64Margin::default()),
            approx_eq!(f64, sum_fits, sum_bl, F64Margin::default())
        );

        // Check this block of floats matches
        assert_eq!(fits_hdu_data, mwalib_hdu_data_by_bl);
    }

    #[test]
    fn test_validate_first_hdu() {
        // Open the test mwax file
        let metafits_filename = "test_files/1101503312_1_timestep/1101503312.metafits";
        let filename =
            "test_files/1101503312_1_timestep/1101503312_20141201210818_gpubox01_00.fits";

        //
        // Read the observation using mwalib
        //
        // Open a context and load in a test metafits and gpubox file
        let gpuboxfiles = vec![filename];
        let mut context = mwalibContext::new(&metafits_filename, &gpuboxfiles)
            .expect("Failed to create mwalibContext");

        let coarse_channel = context.coarse_channels[0].gpubox_number;
        let (batch_index, _) =
            context.gpubox_time_map[&context.timesteps[0].unix_time_ms][&coarse_channel];

        let mut fptr = &mut context.gpubox_batches[batch_index].gpubox_files[0].fptr;

        let result_valid = mwalibContext::validate_first_hdu(
            context.corr_version,
            context.num_fine_channels_per_coarse,
            context.num_baselines,
            context.num_visibility_pols,
            &mut fptr,
        );

        let result_invalid1 = mwalibContext::validate_first_hdu(
            context.corr_version,
            context.num_fine_channels_per_coarse + 1,
            context.num_baselines,
            context.num_visibility_pols,
            &mut fptr,
        );

        let result_invalid2 = mwalibContext::validate_first_hdu(
            context.corr_version,
            context.num_fine_channels_per_coarse,
            context.num_baselines + 1,
            context.num_visibility_pols,
            &mut fptr,
        );

        let result_invalid3 = mwalibContext::validate_first_hdu(
            context.corr_version,
            context.num_fine_channels_per_coarse,
            context.num_baselines,
            context.num_visibility_pols + 1,
            &mut fptr,
        );

        // This is valid
        assert!(result_valid.is_ok());

        assert!(result_invalid1.is_err());

        assert!(result_invalid2.is_err());

        assert!(result_invalid3.is_err());
    }
}
