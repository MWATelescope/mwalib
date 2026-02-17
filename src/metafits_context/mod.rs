// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! The main interface to MWA data.
use chrono::{DateTime, Duration, FixedOffset};
use num_traits::ToPrimitive;
use std::fmt;
use std::path::Path;

use self::error::MetafitsError;
use crate::antenna::*;
use crate::baseline::*;
use crate::coarse_channel::*;
use crate::fits_read::*;
use crate::rfinput::*;
use crate::types::*;
use crate::voltage_files::*;
use crate::*;

pub mod error;

pub(crate) mod ffi;

#[cfg(test)]
pub(crate) mod ffi_test;
#[cfg(test)]
mod test;

#[cfg(any(feature = "python", feature = "python-stubgen"))]
use pyo3::prelude::*;
#[cfg(any(feature = "python", feature = "python-stubgen"))]
mod python;
#[cfg(feature = "python-stubgen")]
use pyo3_stub_gen_derive::gen_stub_pyclass;

///
/// Metafits context. This represents the basic metadata for an MWA observation.
///
#[cfg_attr(feature = "python-stubgen", gen_stub_pyclass)]
#[cfg_attr(
    any(feature = "python", feature = "python-stubgen"),
    pyclass(get_all, set_all, from_py_object)
)]
#[derive(Clone, Debug)]
pub struct MetafitsContext {
    /// mwa version    
    pub mwa_version: Option<MWAVersion>,
    /// Observation id
    pub obs_id: u32,
    /// Scheduled start (gps time) of observation
    pub sched_start_gps_time_ms: u64,
    /// Scheduled end (gps time) of observation
    pub sched_end_gps_time_ms: u64,
    /// Scheduled start (UNIX time) of observation
    pub sched_start_unix_time_ms: u64,
    /// Scheduled end (UNIX time) of observation
    pub sched_end_unix_time_ms: u64,
    /// Scheduled start (UTC) of observation
    pub sched_start_utc: DateTime<FixedOffset>,
    /// Scheduled end (UTC) of observation
    pub sched_end_utc: DateTime<FixedOffset>,
    /// Scheduled start (MJD) of observation
    pub sched_start_mjd: f64,
    /// Scheduled end (MJD) of observation
    pub sched_end_mjd: f64,
    /// Scheduled duration of observation
    pub sched_duration_ms: u64,
    /// DUT1 (i.e. UTC-UT1). The UTC of the obsid is used to determine this
    /// value. Calculated by astropy. Made optional for compatibility.
    pub dut1: Option<f64>,
    /// RA tile pointing
    pub ra_tile_pointing_degrees: f64,
    /// DEC tile pointing
    pub dec_tile_pointing_degrees: f64,
    /// RA phase centre
    pub ra_phase_center_degrees: Option<f64>,
    /// DEC phase centre
    pub dec_phase_center_degrees: Option<f64>,
    /// AZIMUTH of the pointing centre in degrees
    pub az_deg: f64,
    /// ALTITUDE (a.k.a. elevation) of the pointing centre in degrees
    pub alt_deg: f64,
    /// Zenith angle of the pointing centre in degrees
    pub za_deg: f64,
    /// AZIMUTH of the pointing centre in radians
    pub az_rad: f64,
    /// ALTITUDE (a.k.a. elevation) of the pointing centre in radians
    pub alt_rad: f64,
    /// Zenith angle of the pointing centre in radians
    pub za_rad: f64,
    /// Altitude of Sun
    pub sun_alt_deg: Option<f64>,
    /// Distance from pointing center to Sun
    pub sun_distance_deg: Option<f64>,
    /// Distance from pointing center to the Moon
    pub moon_distance_deg: Option<f64>,
    /// Distance from pointing center to Jupiter
    pub jupiter_distance_deg: Option<f64>,
    /// Local Sidereal Time in degrees (at the midpoint of the observation)
    pub lst_deg: f64,
    /// Local Sidereal Time in radians (at the midpoint of the observation)
    pub lst_rad: f64,
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
    pub obs_name: String,
    /// MWA observation mode
    pub mode: MWAMode,
    /// Which Geometric delays have been applied to the data?
    pub geometric_delays_applied: GeometricDelaysApplied,
    /// Have cable delays been applied to the data?
    pub cable_delays_applied: CableDelaysApplied,
    /// Have calibration delays and gains been applied to the data?
    pub calibration_delays_and_gains_applied: bool,
    /// Have signal chain corrections been applied to the data?
    pub signal_chain_corrections_applied: bool,
    /// Correlator fine_chan_resolution
    pub corr_fine_chan_width_hz: u32,
    /// Correlator mode dump time
    pub corr_int_time_ms: u64,
    /// Correlator visibility scaling factor used to get the visibilities in Jansky-like units
    pub corr_raw_scale_factor: f32,
    /// Number of fine channels in each coarse channel for a correlator observation
    pub num_corr_fine_chans_per_coarse: usize,
    /// Voltage fine_chan_resolution
    pub volt_fine_chan_width_hz: u32,
    /// Number of fine channels in each coarse channel for a voltage observation
    pub num_volt_fine_chans_per_coarse: usize,
    /// Array of receiver numbers
    pub receivers: Vec<usize>,
    /// Number of recievers
    pub num_receivers: usize,
    /// Array of beamformer delays
    pub delays: Vec<u32>,
    /// Number of beamformer delays
    pub num_delays: usize,
    /// Intended for calibration
    pub calibrator: bool,
    /// Calibrator source
    pub calibrator_source: String,
    /// ATTEN_DB  // global analogue attenuation, in dB
    pub global_analogue_attenuation_db: f64,
    /// Seconds of bad data after observation starts
    pub quack_time_duration_ms: u64,
    /// OBSID+QUACKTIM as Unix timestamp (first good timestep)
    pub good_time_unix_ms: u64,
    /// Good time expressed in GPS seconds
    pub good_time_gps_ms: u64,
    /// Total number of antennas (tiles) in the array
    pub num_ants: usize,
    /// We also have just the antennas
    pub antennas: Vec<Antenna>,
    /// Total number of rf_inputs (tiles * 2 pols X&Y)
    pub num_rf_inputs: usize,
    /// The Metafits defines an rf chain for antennas(tiles) * pol(X,Y)
    pub rf_inputs: Vec<Rfinput>,
    /// Number of antenna pols. e.g. X and Y
    pub num_ant_pols: usize,
    /// Number of timesteps defined in the metafits file
    pub num_metafits_timesteps: usize,
    /// Vector of timesteps based on the metafits file
    pub metafits_timesteps: Vec<TimeStep>,
    /// Number of coarse channels based on the metafits file
    pub num_metafits_coarse_chans: usize,
    /// Vector of coarse channels based on the metafits file
    pub metafits_coarse_chans: Vec<CoarseChannel>,
    /// Number of fine channels for the whole observation
    pub num_metafits_fine_chan_freqs: usize,
    /// Vector of fine channel frequencies for the whole observation
    pub metafits_fine_chan_freqs_hz: Vec<f64>,
    /// Total bandwidth of observation assuming we have all coarse channels
    pub obs_bandwidth_hz: u32,
    /// Bandwidth of each coarse channel
    pub coarse_chan_width_hz: u32,
    /// The value of the FREQCENT key in the metafits file, but in Hz.
    pub centre_freq_hz: u32,
    /// Number of baselines stored. This is autos plus cross correlations
    pub num_baselines: usize,
    /// Baslines
    pub baselines: Vec<Baseline>,
    /// Number of polarisation combinations in the visibilities e.g. XX,XY,YX,YY == 4
    pub num_visibility_pols: usize,
    /// Filename of the metafits we were given
    pub metafits_filename: String,
    /// Was this observation using oversampled coarse channels?
    pub oversampled: bool,
    /// Was deripple applied to this observation?
    pub deripple_applied: bool,
    /// What was the configured deripple_param?
    /// If deripple_applied is False then this deripple param was not applied
    pub deripple_param: String,
    /// Best calibration fit ID
    pub best_cal_fit_id: Option<u32>,
    /// Best calibration observation ID
    pub best_cal_obs_id: Option<u32>,
    /// Best calibration fit code version
    pub best_cal_code_ver: Option<String>,
    /// Best calibration fit timestamp
    pub best_cal_fit_timestamp: Option<String>,
    /// Best calibration fit creator
    pub best_cal_creator: Option<String>,
    /// Best calibration fit iterations
    pub best_cal_fit_iters: Option<u16>,
    /// Best calibration fit iteration limit
    pub best_cal_fit_iter_limit: Option<u16>,
    /// Signal Chain corrections
    pub signal_chain_corrections: Option<Vec<SignalChainCorrection>>,
    /// Number of Signal Chain corrections
    pub num_signal_chain_corrections: usize,
    /// Calibration fits
    pub calibration_fits: Option<Vec<CalibrationFit>>,
    /// Number of calibration fits
    pub num_calibration_fits: usize,
    /// Beamformer beams
    pub metafits_voltage_beams: Option<Vec<VoltageBeam>>,
    /// Number of beamformer beams defined in this observation
    pub num_metafits_voltage_beams: usize,
    /// Number of coherent beamformer beams defined in this observation
    pub num_metafits_coherent_beams: usize,
    /// Number of incoherent beamformer beams defined in this observation
    pub num_metafits_incoherent_beams: usize,
}

impl MetafitsContext {
    /// From a path to a metafits file, create a `MetafitsContext`.
    ///
    /// # Arguments
    ///
    /// * `metafits_filename` - filename of metafits file as a path or string.
    ///
    /// * `mwa_version` - an Option containing the MWA version the metafits should be interpreted as. Pass None to have mwalib guess based on the MODE in the metafits.
    ///
    /// # Returns
    ///
    /// * Result containing a populated MetafitsContext object if Ok.
    ///
    pub fn new<P: AsRef<Path>>(
        metafits: P,
        mwa_version: Option<MWAVersion>,
    ) -> Result<Self, MwalibError> {
        // Check CFITSIO is reentrant before proceeding
        if !fits_read::is_fitsio_reentrant() {
            return Err(MwalibError::Fits(FitsError::CfitsioIsNotReentrant));
        }

        Self::new_inner(metafits.as_ref(), mwa_version)
    }

    fn new_inner(metafits: &Path, mwa_version: Option<MWAVersion>) -> Result<Self, MwalibError> {
        // Call the internal new metafits method
        let mut new_context = MetafitsContext::new_internal(metafits)?;

        // determine mwa_version if None was passed in
        new_context.mwa_version = match mwa_version {
            None => match new_context.mode {
                MWAMode::Hw_Lfiles => Some(MWAVersion::CorrLegacy),
                MWAMode::Voltage_Start | MWAMode::Voltage_Buffer => {
                    Some(MWAVersion::VCSLegacyRecombined)
                }
                MWAMode::Mwax_Correlator => Some(MWAVersion::CorrMWAXv2),
                MWAMode::Mwax_Vcs | MWAMode::Mwax_Buffer => Some(MWAVersion::VCSMWAXv2),
                MWAMode::Mwax_Beamformer => Some(MWAVersion::BeamformerMWAXv2),
                MWAMode::Mwax_Corr_Bf => Some(MWAVersion::CorrBeamformerMWAXv2),
                _ => {
                    return Err(MwalibError::Metafits(
                        MetafitsError::UnableToDetermineMWAVersionFromMode(new_context.mode),
                    ))
                }
            },
            m => m,
        };

        // The rf inputs should be sorted depending on the Version
        if matches!(
            new_context.mwa_version,
            Some(MWAVersion::VCSLegacyRecombined)
        ) {
            new_context.rf_inputs.sort_by_key(|k| k.vcs_order);
        }

        // Update the voltage fine channel size now that we know which mwaversion we are using
        if new_context.mwa_version == Some(MWAVersion::VCSMWAXv2) {
            // MWAX VCS- the data is unchannelised so coarse chan width == fine chan width
            new_context.volt_fine_chan_width_hz = new_context.coarse_chan_width_hz;
            new_context.num_volt_fine_chans_per_coarse = 1;
        }

        // Populate the coarse channels
        new_context.populate_expected_coarse_channels(new_context.mwa_version.unwrap())?;

        // Now populate the fine channels
        new_context.metafits_fine_chan_freqs_hz = CoarseChannel::get_fine_chan_centres_array_hz(
            new_context.mwa_version.unwrap(),
            &new_context.metafits_coarse_chans,
            match new_context.mwa_version.unwrap() {
                MWAVersion::VCSLegacyRecombined | MWAVersion::VCSMWAXv2 => {
                    new_context.volt_fine_chan_width_hz
                }
                MWAVersion::CorrLegacy
                | MWAVersion::CorrOldLegacy
                | MWAVersion::CorrMWAXv2
                | MWAVersion::CorrBeamformerMWAXv2
                | MWAVersion::BeamformerMWAXv2 => new_context.corr_fine_chan_width_hz,
            },
            match new_context.mwa_version.unwrap() {
                MWAVersion::VCSLegacyRecombined | MWAVersion::VCSMWAXv2 => {
                    new_context.num_volt_fine_chans_per_coarse
                }
                MWAVersion::CorrLegacy
                | MWAVersion::CorrOldLegacy
                | MWAVersion::CorrMWAXv2
                | MWAVersion::CorrBeamformerMWAXv2
                | MWAVersion::BeamformerMWAXv2 => new_context.num_corr_fine_chans_per_coarse,
            },
        );
        new_context.num_metafits_fine_chan_freqs = new_context.metafits_fine_chan_freqs_hz.len();

        // Populate the timesteps
        new_context.populate_expected_timesteps(new_context.mwa_version.unwrap())?;

        // Return the new context
        Ok(new_context)
    }

    /// From a path to a metafits file, create a `MetafitsContext`.
    ///
    /// # Arguments
    ///
    /// * `metafits_filename` - filename of metafits file as a path or string.
    ///
    ///
    /// # Returns
    ///
    /// * Result containing a populated MetafitsContext object if Ok.
    ///
    ///
    pub(crate) fn new_internal<T: AsRef<std::path::Path>>(
        metafits: T,
    ) -> Result<Self, MwalibError> {
        // Pull out observation details. Save the metafits HDU for faster
        // accesses.
        let metafits_filename = metafits
            .as_ref()
            .to_str()
            .expect("Metafits filename is not UTF-8 compliant")
            .to_string();

        let mut metafits_fptr = fits_open!(&metafits)?;
        let metafits_hdu = fits_open_hdu!(&mut metafits_fptr, 0)?;
        let metafits_tile_table_hdu = fits_open_hdu_by_name!(&mut metafits_fptr, "TILEDATA")?;

        // The calibration HDU is not always present, so we will check the result when we need to use it.
        // is_ok() means it exists, is_err() = True means it does not.
        let metafits_cal_hdu_result = fits_open_hdu_by_name!(&mut metafits_fptr, "CALIBDATA");

        // The signal chain HDU is not always present, so we will check the result when we need to use it.
        // is_ok() means it exists, is_err() = True means it does not.
        let metafits_signalchain_hdu_result =
            fits_open_hdu_by_name!(&mut metafits_fptr, "SIGCHAINDATA");

        // The voltage beams HDU is not always present, so we will check the result when we need to use it.
        // is_ok() means it exists, is_err() = True means it does not.
        let metafits_beams_hdu_result = fits_open_hdu_by_name!(&mut metafits_fptr, "VOLTAGEBEAMS");

        // Populate obsid from the metafits
        let obsid = get_required_fits_key!(&mut metafits_fptr, &metafits_hdu, "GPSTIME")?;

        // oversampled not garaunteed to be in the metafits. Default to False
        let oversampled: bool = matches!(
            get_optional_fits_key!(&mut metafits_fptr, &metafits_hdu, "OVERSAMP")?.unwrap_or(0),
            1
        );

        // from MWA_Tools/CONV2UVFITS/convutils.h
        // Used to determine electrical lengths if EL_ not present in metafits for an rf_input
        let quack_time_duration_ms: u64 = {
            let qt: f64 = get_required_fits_key!(&mut metafits_fptr, &metafits_hdu, "QUACKTIM")?;
            (qt * 1000.).round() as _
        };
        let good_time_unix_ms: u64 = {
            let gt: f64 = get_required_fits_key!(&mut metafits_fptr, &metafits_hdu, "GOODTIME")?;
            (gt * 1000.).round() as _
        };

        // observation bandwidth (read from metafits in MHz)
        let metafits_observation_bandwidth_hz: u32 = {
            let bw: f64 = get_required_fits_key!(&mut metafits_fptr, &metafits_hdu, "BANDWDTH")?;
            (bw * 1e6).round() as _
        };

        // Populate coarse channels
        // Get metafits info
        let (metafits_coarse_chan_vec, metafits_coarse_chan_width_hz) =
            CoarseChannel::get_metafits_coarse_channel_info(
                &mut metafits_fptr,
                &metafits_hdu,
                metafits_observation_bandwidth_hz,
            )?;

        // Populate an empty vector for the coarse channels until we know the MWAVersion
        // This is because the coarse channel vector will be different depending on the MWAVersion
        let metafits_coarse_chans: Vec<CoarseChannel> =
            Vec::with_capacity(metafits_coarse_chan_vec.len());
        let num_metafits_coarse_chans: usize = 0;

        // Create a vector of rf_input structs from the metafits
        let num_rf_inputs: usize =
            get_required_fits_key!(&mut metafits_fptr, &metafits_hdu, "NINPUTS")?;

        // There are twice as many inputs as
        // there are antennas; halve that value.
        let num_antennas = num_rf_inputs / 2;

        // Signal chain corrections
        let signal_chain_corrections: Option<Vec<SignalChainCorrection>>;
        let num_signal_chain_corrections: usize;

        match metafits_signalchain_hdu_result {
            Ok(metafits_signalchain_hdu) => {
                let sig_chain_corrs = signal_chain_correction::populate_signal_chain_corrections(
                    &mut metafits_fptr,
                    &metafits_signalchain_hdu,
                )?;

                num_signal_chain_corrections = sig_chain_corrs.len();
                signal_chain_corrections = Some(sig_chain_corrs);
            }
            Err(_) => {
                // This will occur if the HDU does not exist (old / or metafits without this)
                num_signal_chain_corrections = 0;
                signal_chain_corrections = None;
            }
        }

        // Create a vector of rf_input structs from the metafits
        let mut rf_inputs: Vec<Rfinput> = Rfinput::populate_rf_inputs(
            num_rf_inputs,
            &mut metafits_fptr,
            &metafits_tile_table_hdu,
            MWALIB_MWA_COAX_V_FACTOR,
            metafits_coarse_chan_vec.len(),
            &signal_chain_corrections,
        )?;
        assert_eq!(num_rf_inputs, rf_inputs.len());

        // Sort the rf_inputs back into the correct output order
        rf_inputs.sort_by_key(|k| k.subfile_order);

        // Now populate the antennas (note they need to be sorted by subfile_order)
        let antennas: Vec<Antenna> = Antenna::populate_antennas(&rf_inputs);

        // Always assume that MWA antennas have 2 pols
        let num_antenna_pols = 2;

        // Populate baselines
        let baselines = Baseline::populate_baselines(num_antennas);

        // Populate the pols that come out of the correlator
        let num_visibility_pols = 4; // no easy way to get the count of enum variants

        // `num_baselines` is the number of cross-correlations + the number of
        // auto-correlations.
        let num_baselines = (num_antennas * (num_antennas + 1)) / 2;

        // The FREQCENT value in the metafits is in units of kHz - make it Hz.
        let centre_freq_hz: u32 = {
            let cf: f64 = get_required_fits_key!(&mut metafits_fptr, &metafits_hdu, "FREQCENT")?;
            (cf * 1e6).round() as _
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
        let scheduled_duration_ms: u64 = {
            let ex: u64 = get_required_fits_key!(&mut metafits_fptr, &metafits_hdu, "EXPOSURE")?;
            ex * 1000
        };

        let dut1: Option<f64> = get_optional_fits_key!(&mut metafits_fptr, &metafits_hdu, "DUT1")?;

        let num_metafits_timesteps: usize = 0;
        let metafits_timesteps: Vec<TimeStep> = Vec::new();

        let scheduled_end_utc = scheduled_start_utc
            + Duration::try_milliseconds(scheduled_duration_ms as i64)
                .expect("scheduled_duration_ms is out of range");

        // To increment the mjd we need to fractional proportion of the day that the duration represents
        let scheduled_end_mjd =
            scheduled_start_mjd + (scheduled_duration_ms as f64 / 1000. / 86400.);

        let scheduled_start_gpstime_ms: u64 = obsid as u64 * 1000;
        let scheduled_end_gpstime_ms: u64 = scheduled_start_gpstime_ms + scheduled_duration_ms;

        let scheduled_start_unix_time_ms: u64 = good_time_unix_ms - quack_time_duration_ms;
        let scheduled_end_unix_time_ms: u64 = scheduled_start_unix_time_ms + scheduled_duration_ms;

        let good_time_gps_ms: u64 = scheduled_start_gpstime_ms + quack_time_duration_ms;

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
        let zenith_angle_degrees: f64 = 90.0 - altitude_degrees;
        let sun_altitude_degrees: Option<f64> =
            get_optional_fits_key!(&mut metafits_fptr, &metafits_hdu, "SUN-ALT")?;
        let sun_distance_degrees: Option<f64> =
            get_optional_fits_key!(&mut metafits_fptr, &metafits_hdu, "SUN-DIST")?;
        let moon_distance_degrees: Option<f64> =
            get_optional_fits_key!(&mut metafits_fptr, &metafits_hdu, "MOONDIST")?;
        let jupiter_distance_degrees: Option<f64> =
            get_optional_fits_key!(&mut metafits_fptr, &metafits_hdu, "JUP-DIST")?;
        let lst_degrees: f64 = get_required_fits_key!(&mut metafits_fptr, &metafits_hdu, "LST")?;
        let hour_angle_string = get_required_fits_key!(&mut metafits_fptr, &metafits_hdu, "HA")?;
        let grid_name: String =
            get_optional_fits_key!(&mut metafits_fptr, &metafits_hdu, "GRIDNAME")?
                .unwrap_or_else(|| String::from("NOGRID"));
        let grid_number =
            get_optional_fits_key!(&mut metafits_fptr, &metafits_hdu, "GRIDNUM")?.unwrap_or(0);
        let creator = get_required_fits_key!(&mut metafits_fptr, &metafits_hdu, "CREATOR")?;
        let project_id = get_required_fits_key!(&mut metafits_fptr, &metafits_hdu, "PROJECT")?;
        let observation_name =
            get_required_fits_key!(&mut metafits_fptr, &metafits_hdu, "FILENAME")?;
        let mode: MWAMode = get_required_fits_key!(&mut metafits_fptr, &metafits_hdu, "MODE")?;

        let geometric_delays_applied: GeometricDelaysApplied =
            match get_optional_fits_key!(&mut metafits_fptr, &metafits_hdu, "GEODEL")? {
                Some(g) => match num_traits::FromPrimitive::from_i32(g) {
                    Some(gda) => gda,
                    None => {
                        return Err(MwalibError::Parse {
                            key: String::from("GEODEL"),
                            fits_filename: metafits_filename,
                            hdu_num: 0,
                            source_file: String::from(file!()),
                            source_line: line!(),
                        })
                    }
                },
                None => GeometricDelaysApplied::No,
            };

        let cable_delays_applied: CableDelaysApplied =
            match get_optional_fits_key!(&mut metafits_fptr, &metafits_hdu, "CABLEDEL")? {
                Some(g) => match num_traits::FromPrimitive::from_i32(g) {
                    Some(gda) => gda,
                    None => {
                        return Err(MwalibError::Parse {
                            key: String::from("CABLEDEL"),
                            fits_filename: metafits_filename,
                            hdu_num: 0,
                            source_file: String::from(file!()),
                            source_line: line!(),
                        })
                    }
                },
                None => CableDelaysApplied::NoCableDelaysApplied,
            };

        // This next two keys are specified as TINT not TBOOL in the metafits, so we need to translate 0=false, 1=true
        let calibration_delays_and_gains_applied: bool = matches!(
            (get_optional_fits_key!(&mut metafits_fptr, &metafits_hdu, "CALIBDEL")?).unwrap_or(0),
            1
        );

        let signal_chain_corrections_applied: bool = matches!(
            (get_optional_fits_key!(&mut metafits_fptr, &metafits_hdu, "SIGCHDEL")?).unwrap_or(0),
            1
        );

        // We need to get the correlator integration time
        let integration_time_ms: u64 = {
            let it: f64 = get_required_fits_key!(&mut metafits_fptr, &metafits_hdu, "INTTIME")?;
            (it * 1000.) as _
        };
        let receivers_string: String =
            get_required_fits_key_long_string!(&mut metafits_fptr, &metafits_hdu, "RECVRS")?;

        // This is a new metafits key as of Oct 2021. So assume this value is 1.0 unless it is provided
        let corr_raw_scale_factor: f32 =
            get_optional_fits_key!(&mut metafits_fptr, &metafits_hdu, "RAWSCALE")?.unwrap_or(1.0);

        let receivers: Vec<usize> = receivers_string
            .replace(&['\'', '&'][..], "")
            .split(',')
            .map(|s| s.parse().unwrap())
            .collect();

        let num_receivers = receivers.len();

        let delays_string: String =
            get_required_fits_key!(&mut metafits_fptr, &metafits_hdu, "DELAYS")?;

        let delays: Vec<u32> = delays_string
            .replace(&['\'', '&'][..], "")
            .split(',')
            .map(|s| s.parse().unwrap())
            .collect();

        let num_delays = delays.len();

        // CALIBRAT - defalut to F if not found
        let calibration_string: String =
            get_optional_fits_key!(&mut metafits_fptr, &metafits_hdu, "CALIBRAT")?
                .unwrap_or_else(|| String::from("F"));
        let calibrator: bool = calibration_string == "T";

        // CALIBSRC - default to empty string if not found
        let calibrator_source: String =
            get_optional_fits_key!(&mut metafits_fptr, &metafits_hdu, "CALIBSRC")?
                .unwrap_or_else(|| String::from(""));

        // ATTEN_DB is not garaunteed to be in the metafits. Default to 0
        let global_analogue_attenuation_db: f64 =
            get_optional_fits_key!(&mut metafits_fptr, &metafits_hdu, "ATTEN_DB")?.unwrap_or(0.0);

        // Deripple
        // It is stored as a bool DR_FLAG in older metafits or DERIPPLE in newer ones.
        let dr_flag =
            get_optional_fits_key!(&mut metafits_fptr, &metafits_hdu, "DR_FLAG")?.unwrap_or(0);
        let deripple =
            get_optional_fits_key!(&mut metafits_fptr, &metafits_hdu, "DERIPPLE")?.unwrap_or(0);
        let deripple_applied: bool = dr_flag == 1 || deripple == 1;

        // deripple_param is the type of deripple applied
        let deripple_param: String =
            get_optional_fits_key!(&mut metafits_fptr, &metafits_hdu, "DR_PARAM")?
                .unwrap_or_else(|| String::from(""));

        // Placeholder values- we work these out once we know the mwa_version
        let num_metafits_fine_chan_freqs: usize = 0;
        let metafits_fine_chan_freqs: Vec<f64> = Vec::new();

        // Fine-channel resolution. The FINECHAN value in the metafits is in units
        // of kHz - make it Hz.
        let corr_fine_chan_width_hz: u32 = {
            let fc: f64 = get_required_fits_key!(&mut metafits_fptr, &metafits_hdu, "FINECHAN")?;
            (fc * 1000.).round() as _
        };
        // Determine the number of fine channels per coarse channel.
        let num_corr_fine_chans_per_coarse =
            (metafits_coarse_chan_width_hz / corr_fine_chan_width_hz) as usize;

        // Fine-channel resolution. MWA Legacy is 10 kHz, MWAX is unchannelised
        // For now we specify the Legacy VCS, until we know what type of obs this is when we get the
        // MWAVersion in the more specialised methods which populate the metafits info.
        let volt_fine_chan_width_hz: u32 = 10_000;

        // Determine the number of fine channels per coarse channel.
        let num_volt_fine_chans_per_coarse =
            (metafits_coarse_chan_width_hz / volt_fine_chan_width_hz) as usize;

        // Calibration fits
        let calibration_fits: Option<Vec<CalibrationFit>>;
        let num_calibration_fits: usize;

        match &metafits_cal_hdu_result {
            Ok(metafits_calibdata_hdu) => {
                let cal_fits = calibration_fit::populate_calibration_fits(
                    &mut metafits_fptr,
                    metafits_calibdata_hdu,
                    &rf_inputs,
                    num_metafits_coarse_chans,
                )?;

                num_calibration_fits = cal_fits.len();
                calibration_fits = Some(cal_fits);
            }
            Err(_) => {
                // This will occur if the HDU does not exist (old / or metafits without this)
                num_calibration_fits = 0;
                calibration_fits = None;
            }
        }

        // Handle all of the calibration metadata
        let best_cal_fit_id: Option<u32>;
        let best_cal_obs_id: Option<u32>;
        let best_cal_code_ver: Option<String>;
        let best_cal_fit_timestamp: Option<String>;
        let best_cal_creator: Option<String>;
        let best_cal_fit_iters: Option<u16>;
        let best_cal_fit_iter_limit: Option<u16>;

        match metafits_cal_hdu_result {
            Ok(metafits_cal_hdu) => {
                best_cal_fit_id =
                    get_optional_fits_key!(&mut metafits_fptr, &metafits_cal_hdu, "CALFITID")?;
                best_cal_obs_id =
                    get_optional_fits_key!(&mut metafits_fptr, &metafits_cal_hdu, "CALOBSID")?;
                best_cal_code_ver =
                    get_optional_fits_key!(&mut metafits_fptr, &metafits_cal_hdu, "CALCDVER")?;
                best_cal_fit_timestamp =
                    get_optional_fits_key!(&mut metafits_fptr, &metafits_cal_hdu, "CALDTIME")?;
                best_cal_creator =
                    get_optional_fits_key!(&mut metafits_fptr, &metafits_cal_hdu, "CALCRTR")?;
                best_cal_fit_iters =
                    get_optional_fits_key!(&mut metafits_fptr, &metafits_cal_hdu, "CALITERS")?;

                // Currently this is an f32, but will be changed soon to u16
                // This is basically saying-
                // If we do have the CALITLIM key, get the value and then convert it to a u16.
                // If we don't then just make best_cal_fit_iter_limit = None.
                best_cal_fit_iter_limit =
                    get_optional_fits_key!(&mut metafits_fptr, &metafits_cal_hdu, "CALITLIM")?
                        .and_then(|val: f32| val.to_u16());
            }
            Err(_) => {
                // This will occur if the HDU does not exist (old / or metafits without this)
                best_cal_fit_id = None;
                best_cal_obs_id = None;
                best_cal_code_ver = None;
                best_cal_fit_timestamp = None;
                best_cal_creator = None;
                best_cal_fit_iters = None;
                best_cal_fit_iter_limit = None;
            }
        };

        // voltage beams
        let beams: Option<Vec<VoltageBeam>>;
        let num_beams: usize;
        let num_coherent_beams: usize;
        let num_incoherent_beams: usize;

        match &metafits_beams_hdu_result {
            Ok(metafits_beams_hdu) => {
                let beams_vec = voltage_beam::populate_voltage_beams(
                    &mut metafits_fptr,
                    metafits_beams_hdu,
                    &metafits_coarse_chans,
                    &antennas,
                )?;

                num_beams = beams_vec.len();
                beams = Some(beams_vec);
                num_coherent_beams = beams
                    .as_ref()
                    .unwrap()
                    .iter()
                    .filter(|b| b.coherent == true)
                    .count();
                num_incoherent_beams = beams
                    .as_ref()
                    .unwrap()
                    .iter()
                    .filter(|b| b.coherent == false)
                    .count();
            }
            Err(_) => {
                // This will occur if the HDU does not exist (old / or metafits without this)
                num_beams = 0;
                num_coherent_beams = 0;
                num_incoherent_beams = 0;
                beams = None;
            }
        }

        Ok(MetafitsContext {
            mwa_version: None,
            obs_id: obsid,
            sched_start_gps_time_ms: scheduled_start_gpstime_ms,
            sched_end_gps_time_ms: scheduled_end_gpstime_ms,
            sched_start_unix_time_ms: scheduled_start_unix_time_ms,
            sched_end_unix_time_ms: scheduled_end_unix_time_ms,
            sched_start_utc: scheduled_start_utc,
            sched_end_utc: scheduled_end_utc,
            sched_start_mjd: scheduled_start_mjd,
            sched_end_mjd: scheduled_end_mjd,
            sched_duration_ms: scheduled_duration_ms,
            dut1,
            ra_tile_pointing_degrees,
            dec_tile_pointing_degrees,
            ra_phase_center_degrees,
            dec_phase_center_degrees,
            az_deg: azimuth_degrees,
            alt_deg: altitude_degrees,
            za_deg: zenith_angle_degrees,
            az_rad: azimuth_degrees.to_radians(),
            alt_rad: altitude_degrees.to_radians(),
            za_rad: zenith_angle_degrees.to_radians(),
            sun_alt_deg: sun_altitude_degrees,
            sun_distance_deg: sun_distance_degrees,
            moon_distance_deg: moon_distance_degrees,
            jupiter_distance_deg: jupiter_distance_degrees,
            lst_deg: lst_degrees,
            lst_rad: lst_degrees.to_radians(),
            hour_angle_string,
            grid_name,
            grid_number,
            creator,
            project_id,
            obs_name: observation_name,
            mode,
            geometric_delays_applied,
            cable_delays_applied,
            calibration_delays_and_gains_applied,
            signal_chain_corrections_applied,
            corr_fine_chan_width_hz,
            corr_int_time_ms: integration_time_ms,
            corr_raw_scale_factor,
            num_corr_fine_chans_per_coarse,
            volt_fine_chan_width_hz,
            num_volt_fine_chans_per_coarse,
            receivers,
            num_receivers,
            delays,
            num_delays,
            calibrator,
            calibrator_source,
            global_analogue_attenuation_db,
            quack_time_duration_ms,
            good_time_unix_ms,
            good_time_gps_ms,
            num_ants: num_antennas,
            antennas,
            num_rf_inputs,
            rf_inputs,
            num_ant_pols: num_antenna_pols,
            num_metafits_timesteps,
            metafits_timesteps,
            num_metafits_coarse_chans,
            metafits_coarse_chans,
            num_metafits_fine_chan_freqs,
            metafits_fine_chan_freqs_hz: metafits_fine_chan_freqs,
            obs_bandwidth_hz: metafits_observation_bandwidth_hz,
            coarse_chan_width_hz: metafits_coarse_chan_width_hz,
            centre_freq_hz,
            num_baselines,
            baselines,
            num_visibility_pols,
            metafits_filename,
            oversampled,
            deripple_applied,
            deripple_param,
            best_cal_fit_id,
            best_cal_obs_id,
            best_cal_code_ver,
            best_cal_fit_timestamp,
            best_cal_creator,
            best_cal_fit_iters,
            best_cal_fit_iter_limit,
            signal_chain_corrections,
            num_signal_chain_corrections,
            calibration_fits,
            num_calibration_fits,
            metafits_voltage_beams: beams,
            num_metafits_voltage_beams: num_beams,
            num_metafits_coherent_beams: num_coherent_beams,
            num_metafits_incoherent_beams: num_incoherent_beams,
        })
    }

    /// Given a hint at the expected `MWAVersion`, populate the coarse_channel vector with the expected
    /// coarse channels for an existing populated MetafitsContext.
    ///
    /// # Arguments
    ///
    /// * `mwa_version` - Hint, providing the `MWAVersion` info, so the expected `CoarseChannel`s can be returned.
    ///
    ///
    /// # Returns
    ///
    /// * Result containing ok or an error
    ///
    ///
    pub(crate) fn populate_expected_coarse_channels(
        &mut self,
        mwa_version: MWAVersion,
    ) -> Result<(), MwalibError> {
        // Reopen metafits
        let mut metafits_fptr = fits_open!(&self.metafits_filename)?;
        let metafits_hdu = fits_open_hdu!(&mut metafits_fptr, 0)?;

        // Get metafits info
        let (metafits_coarse_chan_vec, metafits_coarse_chan_width_hz) =
            CoarseChannel::get_metafits_coarse_channel_info(
                &mut metafits_fptr,
                &metafits_hdu,
                self.obs_bandwidth_hz,
            )?;

        // Populate coarse chans from the metafits info.
        self.metafits_coarse_chans
            .extend(CoarseChannel::populate_coarse_channels(
                mwa_version,
                &metafits_coarse_chan_vec,
                metafits_coarse_chan_width_hz,
                None,
                None,
            )?);

        self.num_metafits_coarse_chans = self.metafits_coarse_chans.len();

        Ok(())
    }

    /// Given a hint at the expected `MWAVersion`, populate the timesteps vector with the expected
    /// timesteps for an existing populated MetafitsContext.
    ///
    /// # Arguments
    ///
    /// * `mwa_version` - Hint, providing the `MWAVersion` info, so the expected `TimeStep`s can be returned.
    ///
    ///
    /// # Returns
    ///
    /// * Result containing ok or an error
    ///
    ///
    pub(crate) fn populate_expected_timesteps(
        &mut self,
        mwa_version: MWAVersion,
    ) -> Result<(), MwalibError> {
        // Process the channels based on the gpubox files we have
        self.metafits_timesteps.extend(TimeStep::populate_timesteps(
            self,
            mwa_version,
            self.sched_start_gps_time_ms,
            self.sched_duration_ms,
            self.sched_start_gps_time_ms,
            self.sched_start_unix_time_ms,
        ));

        self.num_metafits_timesteps = self.metafits_timesteps.len();

        Ok(())
    }

    /// Return an expected voltage filenames for the input timestep and coarse channel indices.
    ///
    /// # Arguments
    ///
    /// * `metafits_timestep_index` - the timestep index.
    ///
    /// * `metafits_coarse_chan_index` - the coarse channel index.
    ///
    ///
    /// # Returns
    ///
    /// * Result containing the generated filename or an error
    ///
    ///
    pub fn generate_expected_volt_filename(
        &self,
        metafits_timestep_index: usize,
        metafits_coarse_chan_index: usize,
    ) -> Result<String, VoltageFileError> {
        if metafits_timestep_index >= self.num_metafits_timesteps {
            return Err(VoltageFileError::InvalidTimeStepIndex(
                self.num_metafits_timesteps - 1,
            ));
        }

        if metafits_coarse_chan_index >= self.num_metafits_coarse_chans {
            return Err(VoltageFileError::InvalidCoarseChanIndex(
                self.num_metafits_coarse_chans - 1,
            ));
        }

        // Compose filename
        let obs_id = self.obs_id;
        let gpstime = self.metafits_timesteps[metafits_timestep_index].gps_time_ms / 1000;
        let chan = format!(
            "{:03}",
            self.metafits_coarse_chans[metafits_coarse_chan_index].rec_chan_number
        ); // zero padded to 3 digits

        let out_string = match self.mwa_version.unwrap() {
            MWAVersion::VCSLegacyRecombined => format!("{}_{}_ch{}.dat", obs_id, gpstime, chan),
            MWAVersion::VCSMWAXv2 => format!("{}_{}_{}.sub", obs_id, gpstime, chan),
            _ => {
                return Err(VoltageFileError::InvalidMwaVersion {
                    mwa_version: self.mwa_version.unwrap(),
                })
            }
        };

        Ok(out_string)
    }
}

/// Implements fmt::Display for MetafitsContext struct
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
impl fmt::Display for MetafitsContext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(
            f,
            "MetafitsContext (
    obsid:                     {obsid},
    mode:                      {mode},

    If Correlator Mode:
     fine channel resolution:  {fcw} kHz,
     integration time:         {int_time:.2} s
     num fine channels/coarse: {nfcpc},

    If Voltage Mode:
     fine channel resolution:  {vfcw} kHz,
     num fine channels/coarse: {nvfcpc},

    Geometric delays applied          : {geodel},
    Cable length corrections applied  : {cabledel},
    Calibration delays & gains applied: {calibdel},
    Signal chain corrections applied  : {sigchdel},

    Creator:                  {creator},
    Project ID:               {project_id},
    Observation Name:         {obs_name},
    Receivers:                {receivers},
    Delays:                   {delays},
    Calibration:              {calib},
    Calibrator Source:        {calsrc},
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
    Quack time:               {quack_duration} s,
    Good UNIX start time:     {good_time},

    Num timesteps:            {nts},
    Timesteps:                {ts},

    Num coarse channels:      {ncc},
    Coarse Channels:          {cc},
    Oversampled coarse chans: {os},
    Deripple applied:         {dr} ({dr_param}),

    Num fine channels:        {nfc},
    Fine Channels (MHz):      {fc},

    R.A. (tile_pointing):     {rtpc} degrees,
    Dec. (tile_pointing):     {dtpc} degrees,
    R.A. (phase center):      {rppc} degrees,
    Dec. (phase center):      {dppc} degrees,
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

    num antennas:             {n_ants},
    antennas:                 {ants},
    rf_inputs:                {rfs},

    num antenna pols:         {n_aps},
    num baselines:            {n_bls},
    baselines:                {bl01} v {bl02} to {bll1} v {bll2}
    num auto-correlations:    {n_ants},
    num cross-correlations:   {n_ccs},
    visibility raw scale fact {crsf},

    num visibility pols:      {n_vps},
    visibility pols:          {vp0}, {vp1}, {vp2}, {vp3},

    metafits FREQCENT key:    {freqcent} MHz,

    best calibrator info (if present)
    ..fit_id:                 {bcal_fit_id},
    ..obs_id:                 {bcal_obs_id},
    ..code_ver:               {bcal_code_ver},
    ..fit_timestamp:          {bcal_fit_timestamp},
    ..creator:                {bcal_creator},

    num signal chain corrs:   {n_scc},
    Signal chain corrs:       {scc},

    num calibration fits:     {ncfits},

    metafits filename:        {meta},
)",
            obsid = self.obs_id,
            creator = self.creator,
            project_id = self.project_id,
            obs_name = self.obs_name,
            receivers = pretty_print_vec(&self.receivers, 32),
            delays = pretty_print_vec(&self.delays, 32),
            atten = self.global_analogue_attenuation_db,
            sched_start_unix = self.sched_start_unix_time_ms as f64 / 1e3,
            sched_end_unix = self.sched_end_unix_time_ms as f64 / 1e3,
            sched_start_gps = self.sched_start_gps_time_ms as f64 / 1e3,
            sched_end_gps = self.sched_end_gps_time_ms as f64 / 1e3,
            sched_start_utc = self.sched_start_utc,
            sched_end_utc = self.sched_end_utc,
            sched_start_mjd = self.sched_start_mjd,
            sched_end_mjd = self.sched_end_mjd,
            sched_duration = self.sched_duration_ms as f64 / 1e3,
            quack_duration = self.quack_time_duration_ms as f64 / 1e3,
            good_time = self.good_time_unix_ms as f64 / 1e3,
            ts = pretty_print_vec(&self.metafits_timesteps, 10),
            nts = self.metafits_timesteps.len(),
            cc = pretty_print_vec(&self.metafits_coarse_chans, 24),
            ncc = self.metafits_coarse_chans.len(),
            os = self.oversampled,
            dr = self.deripple_applied,
            dr_param = match self.deripple_applied {
                true => self.deripple_param.to_string(),
                false => String::from("N/A"),
            },
            nfc = self.metafits_fine_chan_freqs_hz.len(),
            fc = pretty_print_vec(
                &self
                    .metafits_fine_chan_freqs_hz
                    .iter()
                    .map(|f| (f / 1000000.) as f32)
                    .collect::<Vec<f32>>(),
                8
            ),
            rtpc = format!("{:.3}", self.ra_tile_pointing_degrees),
            dtpc = format!("{:.3}", self.dec_tile_pointing_degrees),
            rppc = match self.ra_phase_center_degrees {
                Some(s) => format!("{:.3}", s),
                None => String::from("None"),
            },
            dppc = match self.dec_phase_center_degrees {
                Some(s) => format!("{:.3}", s),
                None => String::from("None"),
            },
            az = self.az_deg,
            alt = self.alt_deg,
            sun_alt = match self.sun_alt_deg {
                Some(s) => s.to_string(),
                None => String::from("None"),
            },
            sun_dis = match self.sun_distance_deg {
                Some(s) => s.to_string(),
                None => String::from("None"),
            },
            moon_dis = match self.moon_distance_deg {
                Some(s) => s.to_string(),
                None => String::from("None"),
            },
            jup_dis = match self.jupiter_distance_deg {
                Some(s) => s.to_string(),
                None => String::from("None"),
            },
            lst = self.lst_deg,
            ha = self.hour_angle_string,
            grid = self.grid_name,
            grid_n = self.grid_number,
            calib = self.calibrator,
            calsrc = self.calibrator_source,
            n_ants = self.num_ants,
            ants = pretty_print_vec(&self.antennas, 256),
            rfs = pretty_print_vec(&self.rf_inputs, 10),
            n_aps = self.num_ant_pols,
            n_bls = self.num_baselines,
            bl01 = self.baselines[0].ant1_index,
            bl02 = self.baselines[0].ant2_index,
            bll1 = self.baselines[self.num_baselines - 1].ant1_index,
            bll2 = self.baselines[self.num_baselines - 1].ant2_index,
            n_ccs = self.num_baselines - self.num_ants,
            n_vps = self.num_visibility_pols,
            vp0 = VisPol::XX,
            vp1 = VisPol::XY,
            vp2 = VisPol::YX,
            vp3 = VisPol::YY,
            freqcent = self.centre_freq_hz as f64 / 1e6,
            mode = self.mode,
            geodel = self.geometric_delays_applied,
            cabledel = self.cable_delays_applied,
            calibdel = self.calibration_delays_and_gains_applied,
            sigchdel = self.signal_chain_corrections_applied,
            vfcw = self.volt_fine_chan_width_hz as f64 / 1e3,
            nvfcpc = self.num_volt_fine_chans_per_coarse,
            fcw = self.corr_fine_chan_width_hz as f64 / 1e3,
            nfcpc = self.num_corr_fine_chans_per_coarse,
            int_time = self.corr_int_time_ms as f64 / 1e3,
            crsf = self.corr_raw_scale_factor,
            n_scc = self.num_signal_chain_corrections,
            scc = pretty_print_opt_vec(&self.signal_chain_corrections, 2),
            ncfits = self.num_calibration_fits,
            bcal_fit_id = match self.best_cal_fit_id {
                Some(b) => b.to_string(),
                None => String::from("None"),
            },
            bcal_obs_id = match self.best_cal_obs_id {
                Some(b) => b.to_string(),
                None => String::from("None"),
            },
            bcal_code_ver = match &self.best_cal_code_ver {
                Some(b) => b.to_string(),
                None => String::from("None"),
            },
            bcal_fit_timestamp = match &self.best_cal_fit_timestamp {
                Some(b) => b.to_string(),
                None => String::from("None"),
            },
            bcal_creator = match &self.best_cal_creator {
                Some(b) => b.to_string(),
                None => String::from("None"),
            },
            meta = self.metafits_filename,
        )
    }
}
