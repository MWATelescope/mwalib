// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::{
    antenna::{self},
    baseline, beam, calibration_fit, coarse_channel,
    ffi::{
        ffi_create_c_array, ffi_create_c_string, ffi_free_rust_c_string, ffi_free_rust_struct,
        set_c_string, MWALIB_FAILURE, MWALIB_SUCCESS,
    },
    rfinput, signal_chain_correction, timestep, CableDelaysApplied, CorrelatorContext,
    GeometricDelaysApplied, MWAMode, MWAVersion, MetafitsContext, VoltageContext,
};
use libc::size_t;
use std::ffi::{c_char, CStr};

///
/// This a C struct to allow the caller to consume the metafits metadata
///
#[repr(C)]
pub struct MetafitsMetadata {
    // ---- 8-byte aligned fields (f64 first) ----
    /// ATTEN_DB  // global analogue attenuation, in dB
    pub global_analogue_attenuation_db: f64,
    /// RA tile pointing
    pub ra_tile_pointing_deg: f64,
    /// DEC tile pointing
    pub dec_tile_pointing_deg: f64,
    /// RA phase centre
    pub ra_phase_center_deg: f64,
    /// DEC phase centre
    pub dec_phase_center_deg: f64,
    /// AZIMUTH
    pub az_deg: f64,
    /// ALTITUDE
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
    pub sun_alt_deg: f64,
    /// Distance from pointing center to Sun
    pub sun_distance_deg: f64,
    /// Distance from pointing center to the Moon
    pub moon_distance_deg: f64,
    /// Distance from pointing center to Jupiter
    pub jupiter_distance_deg: f64,
    /// Local Sidereal Time in degrees (at the midpoint of the observation)
    pub lst_deg: f64,
    /// Local Sidereal Time in radians (at the midpoint of the observation)
    pub lst_rad: f64,
    /// Scheduled start (MJD) of observation
    pub sched_start_mjd: f64,
    /// Scheduled end (MJD) of observation
    pub sched_end_mjd: f64,
    /// DUT1 (i.e. UTC-UT1)
    pub dut1: f64,

    // ---- 64-bit integers ----
    /// Correlator mode dump time
    pub corr_int_time_ms: u64,
    /// Scheduled start (UNIX time) of observation
    pub sched_start_unix_time_ms: u64,
    /// Scheduled end (UNIX time) of observation
    pub sched_end_unix_time_ms: u64,
    /// Scheduled start (GPS) of observation
    pub sched_start_gps_time_ms: u64,
    /// Scheduled end (GPS) of observation
    pub sched_end_gps_time_ms: u64,
    /// Scheduled duration of observation
    pub sched_duration_ms: u64,
    /// Seconds of bad data after observation starts
    pub quack_time_duration_ms: u64,
    /// OBSID+QUACKTIM as Unix timestamp (first good timestep)
    pub good_time_unix_ms: u64,
    /// Good time expressed as GPS seconds
    pub good_time_gps_ms: u64,

    // ---- Pointers (8 bytes on 64-bit) ----
    /// Hour Angle of pointing center (as a string)
    pub hour_angle_string: *mut c_char,
    /// GRIDNAME
    pub grid_name: *mut c_char,
    /// CREATOR
    pub creator: *mut c_char,
    /// PROJECT
    pub project_id: *mut c_char,
    /// Observation name
    pub obs_name: *mut c_char,
    /// Calibrator source
    pub calibrator_source: *mut c_char,
    /// filename of metafits file used
    pub metafits_filename: *mut c_char,
    /// What was the configured deripple_param?
    pub deripple_param: *mut c_char,
    /// Best calibration fit code version
    pub best_cal_code_ver: *mut c_char,
    /// Best calibration fit timestamp
    pub best_cal_fit_timestamp: *mut c_char,
    /// Best calibration fit creator
    pub best_cal_creator: *mut c_char,
    /// Array of receiver numbers
    pub receivers: *mut usize,
    /// Array of beamformer delays
    pub delays: *mut u32,
    /// Array of antennas
    pub antennas: *mut antenna::ffi::Antenna,
    /// Array of rf inputs
    pub rf_inputs: *mut rfinput::ffi::Rfinput,
    /// Baseline array
    pub baselines: *mut baseline::ffi::Baseline,
    /// metafits_coarse_chans array
    pub metafits_coarse_chans: *mut coarse_channel::ffi::CoarseChannel,
    /// Vector of fine channel frequencies for the whole observation
    pub metafits_fine_chan_freqs_hz: *mut f64,
    /// metafits_timesteps array
    pub metafits_timesteps: *mut timestep::ffi::TimeStep,
    /// Signal Chain corrections array
    pub signal_chain_corrections: *mut signal_chain_correction::ffi::SignalChainCorrection,
    /// Calibration fits
    pub calibration_fits: *mut calibration_fit::ffi::CalibrationFit,
    /// Beamformer beams array
    pub metafits_beams: *mut beam::ffi::Beam,

    // ---- 32-bit integers ----
    /// Observation id
    pub obs_id: u32,
    /// Correlator fine_chan_resolution
    pub corr_fine_chan_width_hz: u32,
    /// Voltage fine_chan_resolution
    pub volt_fine_chan_width_hz: u32,
    /// Total bandwidth of observation
    pub obs_bandwidth_hz: u32,
    /// Bandwidth of each coarse channel
    pub coarse_chan_width_hz: u32,
    /// Centre frequency of observation
    pub centre_freq_hz: u32,
    /// Best calibration fit ID
    pub best_cal_fit_id: u32,
    /// Best calibration observation ID
    pub best_cal_obs_id: u32,
    /// GRIDNUM
    pub grid_number: i32,

    // ---- 16-bit integers ----
    /// Best calibration fit iterations
    pub best_cal_fit_iters: u16,
    /// Best calibration fit iteration limit
    pub best_cal_fit_iter_limit: u16,

    // ---- Floats smaller than f64 ----
    /// Correlator visibility scaling factor
    pub corr_raw_scale_factor: f32,

    // ---- usize counts ----
    pub num_corr_fine_chans_per_coarse: usize,
    pub num_volt_fine_chans_per_coarse: usize,
    pub num_receivers: usize,
    pub num_delays: usize,
    pub num_ants: usize,
    pub num_rf_inputs: usize,
    pub num_ant_pols: usize,
    pub num_baselines: usize,
    pub num_visibility_pols: usize,
    pub num_metafits_beams: usize,
    pub num_metafits_coherent_beams: usize,
    pub num_metafits_incoherent_beams: usize,
    pub num_metafits_coarse_chans: usize,
    pub num_metafits_fine_chan_freqs_hz: usize,
    pub num_metafits_timesteps: usize,
    pub num_signal_chain_corrections: usize,
    pub num_calibration_fits: usize,

    // ---- Enums ----
    /// mwa version
    pub mwa_version: MWAVersion,
    /// MWA observation mode
    pub mode: MWAMode,
    /// Which Geometric delays have been applied to the data
    pub geometric_delays_applied: GeometricDelaysApplied,
    /// Have cable delays been applied to the data?
    pub cable_delays_applied: CableDelaysApplied,

    // ---- Booleans ----
    /// Have calibration delays and gains been applied to the data?
    pub calibration_delays_and_gains_applied: bool,
    /// Intended for calibration
    pub calibrator: bool,
    /// Was this observation using oversampled coarse channels?
    pub oversampled: bool,
    /// Was deripple applied to this observation?
    pub deripple_applied: bool,

    // ---- time_t fields (usually 8 bytes on 64-bit) ----
    /// Scheduled start (UTC) of observation as a time_t
    pub sched_start_utc: libc::time_t,
    /// Scheduled end (UTC) of observation as a time_t
    pub sched_end_utc: libc::time_t,
}

/// This passed back a struct containing the `MetafitsContext` metadata, given a MetafitsContext, CorrelatorContext or VoltageContext
///
/// # Arguments
///
/// * `metafits_context_ptr` - pointer to an already populated `MetafitsContext` object. (Exclusive with correlator_context_ptr and voltage_context_ptr)
///
/// * `correlator_context_ptr` - pointer to an already populated `CorrelatorContext` object. (Exclusive with metafits_context_ptr and voltage_context_ptr)
///
/// * `voltage_context_ptr` - pointer to an already populated `VoltageContext` object. (Exclusive with metafits_context_ptr and correlator_context_ptr)
///
/// * `out_metafits_metadata_ptr` - pointer to a Rust-owned `mwalibMetafitsMetadata` struct. Free with `mwalib_metafits_metadata_free`
///
/// * `error_message` - pointer to already allocated buffer for any error messages to be returned to the caller.
///
/// * `error_message_length` - length of error_message char* buffer.
///
///
/// # Returns
///
/// * MWALIB_SUCCESS on success, non-zero on failure
///
///
/// # Safety
/// * `error_message` *must* point to an already allocated char* buffer for any error messages.
/// * `metafits_context_ptr` must point to a populated MetafitsContext object from the `mwalib_metafits_context_new` function OR
/// * `correlator_context_ptr` must point to a populated CorrelatorContext object from the 'mwalib_correlator_context_new' function OR
/// * `voltage_context_ptr` must point to a populated VoltageContext object from the `mwalib_voltage_context_new` function. (Set the unused contexts to NULL).
/// * Caller must call `mwalib_metafits_metadata_free` once finished, to free the rust memory.
#[no_mangle]
pub unsafe extern "C" fn mwalib_metafits_metadata_get(
    metafits_context_ptr: *mut MetafitsContext,
    correlator_context_ptr: *mut CorrelatorContext,
    voltage_context_ptr: *mut VoltageContext,
    out_metafits_metadata_ptr: &mut *mut MetafitsMetadata,
    error_message: *mut c_char,
    error_message_length: size_t,
) -> i32 {
    // Ensure only either metafits XOR correlator XOR voltage context is passed in
    let provided = usize::from(!metafits_context_ptr.is_null())
        + usize::from(!correlator_context_ptr.is_null())
        + usize::from(!voltage_context_ptr.is_null());
    if provided != 1 {
        set_c_string(
            "mwalib_metafits_metadata_get() ERROR: pointers for metafits_context_ptr, correlator_context_ptr and/or voltage_context_ptr were passed in. Only one should be provided.",
            error_message as *mut c_char,
            error_message_length,
        );
        return MWALIB_FAILURE;
    }

    // Create our metafits context pointer depending on what was passed in
    let metafits_context = if !metafits_context_ptr.is_null() {
        // Caller passed in a metafits context, so use that
        &*metafits_context_ptr
    } else if !correlator_context_ptr.is_null() {
        // Caller passed in a correlator context, so use that
        &(*correlator_context_ptr).metafits_context
    } else {
        // Caller passed in a voltage context, so use that
        &(*voltage_context_ptr).metafits_context
    };

    //
    // Populate members
    //

    // Populate antennas
    let (antennas_ptr, antennas_len) = antenna::ffi::Antenna::populate_array(&metafits_context);

    // Populate baselines
    let (baselines_ptr, baselines_len) = baseline::ffi::Baseline::populate_array(&metafits_context);

    // Populate beams
    let (beams_ptr, beams_len) = beam::ffi::Beam::populate_array(metafits_context);

    // Populate calibration fits
    let (calibration_fits_ptr, calibration_fits_len) =
        calibration_fit::ffi::CalibrationFit::populate_array(metafits_context);

    // Populate metafits coarse channels
    let (coarse_channels_ptr, coarse_channels_len) =
        coarse_channel::ffi::CoarseChannel::populate_array(metafits_context);

    // Populate metafits timesteps
    let (timesteps_ptr, timesteps_len) = timestep::ffi::TimeStep::populate_array(metafits_context);

    // Populate rf_inputs
    let (rfinputs_ptr, rfinputs_len) = rfinput::ffi::Rfinput::populate_array(metafits_context);

    // Populate signal chain corrections
    let (signal_chain_corrections_ptr, signal_chain_corrections_len) =
        signal_chain_correction::ffi::SignalChainCorrection::populate_array(metafits_context);

    //
    // Populate primitive arrays
    //

    // Populate receivers
    let (receivers_ptr, receivers_len) = ffi_create_c_array(metafits_context.receivers.clone());

    // populate beamformer delays
    let (delays_ptr, delays_len) = ffi_create_c_array(metafits_context.delays.clone());

    // populate metafits_fine_chan_freqs_hz
    let (metafits_fine_chan_freqs_hz_ptr, metafits_fine_chan_freqs_hz_len) =
        ffi_create_c_array(metafits_context.metafits_fine_chan_freqs_hz.clone());

    // Populate the outgoing structure with data from the metafits context
    // We explicitly break out the attributes so at compile time it will let us know
    // if there have been new fields added to the rust struct, then we can choose to
    // ignore them (with _) or add that field to the FFI struct.
    let out_metadata = {
        let MetafitsContext {
            mwa_version,
            obs_id,
            sched_start_gps_time_ms,
            sched_end_gps_time_ms,
            sched_start_unix_time_ms,
            sched_end_unix_time_ms,
            sched_start_utc,
            sched_end_utc,
            sched_start_mjd,
            sched_end_mjd,
            sched_duration_ms,
            dut1,
            ra_tile_pointing_degrees,
            dec_tile_pointing_degrees,
            ra_phase_center_degrees,
            dec_phase_center_degrees,
            az_deg,
            alt_deg,
            za_deg,
            az_rad,
            alt_rad,
            za_rad,
            sun_alt_deg,
            sun_distance_deg,
            moon_distance_deg,
            jupiter_distance_deg,
            lst_deg: lst_degrees,
            lst_rad: lst_radians,
            hour_angle_string,
            grid_name,
            grid_number,
            creator,
            project_id,
            obs_name,
            mode,
            geometric_delays_applied,
            cable_delays_applied,
            calibration_delays_and_gains_applied,
            corr_fine_chan_width_hz,
            corr_int_time_ms,
            corr_raw_scale_factor,
            num_corr_fine_chans_per_coarse,
            volt_fine_chan_width_hz,
            num_volt_fine_chans_per_coarse,
            receivers: _,
            num_receivers: _,
            delays: _,
            num_delays: _,
            calibrator,
            calibrator_source,
            global_analogue_attenuation_db,
            quack_time_duration_ms,
            good_time_unix_ms,
            good_time_gps_ms,
            num_ants: _,
            antennas: _, // This is populated seperately
            num_rf_inputs: _,
            rf_inputs: _, // This is populated seperately
            num_ant_pols,
            num_metafits_timesteps: _,
            metafits_timesteps: _, // This is populated seperately
            num_metafits_coarse_chans: _,
            metafits_coarse_chans: _, // This is populated seperately
            num_metafits_fine_chan_freqs: _,
            metafits_fine_chan_freqs_hz: _,
            obs_bandwidth_hz,
            coarse_chan_width_hz,
            centre_freq_hz,
            num_baselines: _,
            baselines: _, // This is populated seperately
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
            signal_chain_corrections: _, // This is populated seperately
            num_signal_chain_corrections: _,
            calibration_fits: _, // This is populated seperately
            num_calibration_fits: _,
            metafits_beams: _, // This is populated seperately
            num_metafits_beams: _,
            num_metafits_coherent_beams,
            num_metafits_incoherent_beams,
        } = metafits_context;

        MetafitsMetadata {
            mwa_version: mwa_version.unwrap(),
            obs_id: *obs_id,
            global_analogue_attenuation_db: *global_analogue_attenuation_db,
            ra_tile_pointing_deg: *ra_tile_pointing_degrees,
            dec_tile_pointing_deg: *dec_tile_pointing_degrees,
            ra_phase_center_deg: (*ra_phase_center_degrees).unwrap_or(f64::NAN),
            dec_phase_center_deg: (*dec_phase_center_degrees).unwrap_or(f64::NAN),
            az_deg: *az_deg,
            alt_deg: *alt_deg,
            za_deg: *za_deg,
            az_rad: *az_rad,
            alt_rad: *alt_rad,
            za_rad: *za_rad,
            sun_alt_deg: (*sun_alt_deg).unwrap_or(f64::NAN),
            sun_distance_deg: (*sun_distance_deg).unwrap_or(f64::NAN),
            moon_distance_deg: (*moon_distance_deg).unwrap_or(f64::NAN),
            jupiter_distance_deg: (*jupiter_distance_deg).unwrap_or(f64::NAN),
            lst_deg: *lst_degrees,
            lst_rad: *lst_radians,
            hour_angle_string: ffi_create_c_string(hour_angle_string),
            grid_name: ffi_create_c_string(grid_name),
            grid_number: *grid_number,
            creator: ffi_create_c_string(creator),
            project_id: ffi_create_c_string(project_id),
            obs_name: ffi_create_c_string(obs_name),
            mode: *mode,
            geometric_delays_applied: *geometric_delays_applied,
            cable_delays_applied: *cable_delays_applied,
            calibration_delays_and_gains_applied: *calibration_delays_and_gains_applied,
            corr_fine_chan_width_hz: *corr_fine_chan_width_hz,
            corr_int_time_ms: *corr_int_time_ms,
            corr_raw_scale_factor: *corr_raw_scale_factor,
            num_corr_fine_chans_per_coarse: *num_corr_fine_chans_per_coarse,
            volt_fine_chan_width_hz: *volt_fine_chan_width_hz,
            num_volt_fine_chans_per_coarse: *num_volt_fine_chans_per_coarse,
            receivers: receivers_ptr,
            num_receivers: receivers_len,
            delays: delays_ptr,
            num_delays: delays_len,
            calibrator: *calibrator,
            calibrator_source: ffi_create_c_string(calibrator_source),
            sched_start_utc: sched_start_utc.timestamp(),
            sched_end_utc: sched_end_utc.timestamp(),
            sched_start_mjd: *sched_start_mjd,
            sched_end_mjd: *sched_end_mjd,
            sched_start_unix_time_ms: *sched_start_unix_time_ms,
            sched_end_unix_time_ms: *sched_end_unix_time_ms,
            sched_start_gps_time_ms: *sched_start_gps_time_ms,
            sched_end_gps_time_ms: *sched_end_gps_time_ms,
            sched_duration_ms: *sched_duration_ms,
            dut1: dut1.unwrap_or(0.0),
            quack_time_duration_ms: *quack_time_duration_ms,
            good_time_unix_ms: *good_time_unix_ms,
            good_time_gps_ms: *good_time_gps_ms,
            num_ants: antennas_len,
            antennas: antennas_ptr,
            num_rf_inputs: rfinputs_len,
            rf_inputs: rfinputs_ptr,
            num_ant_pols: *num_ant_pols,
            num_baselines: baselines_len,
            baselines: baselines_ptr,
            num_visibility_pols: *num_visibility_pols,
            num_metafits_coarse_chans: coarse_channels_len,
            metafits_coarse_chans: coarse_channels_ptr,
            num_metafits_fine_chan_freqs_hz: metafits_fine_chan_freqs_hz_len,
            metafits_fine_chan_freqs_hz: metafits_fine_chan_freqs_hz_ptr,
            num_metafits_timesteps: timesteps_len,
            metafits_timesteps: timesteps_ptr,
            obs_bandwidth_hz: *obs_bandwidth_hz,
            coarse_chan_width_hz: *coarse_chan_width_hz,
            centre_freq_hz: *centre_freq_hz,
            metafits_filename: ffi_create_c_string(metafits_filename),
            oversampled: *oversampled,
            deripple_applied: *deripple_applied,
            deripple_param: ffi_create_c_string(deripple_param),
            best_cal_fit_id: best_cal_fit_id.unwrap_or_default(),
            best_cal_obs_id: best_cal_obs_id.unwrap_or_default(),
            best_cal_code_ver: ffi_create_c_string(
                best_cal_code_ver.as_deref().unwrap_or_default(),
            ),
            best_cal_fit_timestamp: ffi_create_c_string(
                best_cal_fit_timestamp.as_deref().unwrap_or_default(),
            ),
            best_cal_creator: ffi_create_c_string(best_cal_creator.as_deref().unwrap_or_default()),
            best_cal_fit_iters: best_cal_fit_iters.unwrap_or_default(),
            best_cal_fit_iter_limit: best_cal_fit_iter_limit.unwrap_or_default(),
            signal_chain_corrections: signal_chain_corrections_ptr,
            num_signal_chain_corrections: signal_chain_corrections_len,
            calibration_fits: calibration_fits_ptr,
            num_calibration_fits: calibration_fits_len,
            num_metafits_beams: beams_len,
            metafits_beams: beams_ptr,
            num_metafits_coherent_beams: *num_metafits_coherent_beams,
            num_metafits_incoherent_beams: *num_metafits_incoherent_beams,
        }
    };

    // Pass back a pointer to the rust owned struct
    *out_metafits_metadata_ptr = Box::into_raw(Box::new(out_metadata));

    // Return Success
    MWALIB_SUCCESS
}

/// Free a previously-allocated `mwalibMetafitsMetadata` struct.
///
/// # Arguments
///
/// * `metafits_metadata_ptr` - pointer to an already populated `mwalibMetafitsMetadata` object
///
///
/// # Returns
///
/// * MWALIB_SUCCESS on success, non-zero on failure
///
///
/// # Safety
/// * This must be called once caller is finished with the `mwalibMetafitsMetadata` object
/// * `metafits_metadata_ptr` must point to a populated `mwalibMetafitsMetadata` object from the `mwalib_metafits_metadata_get` function.
/// * `metafits_metadata_ptr` must not have already been freed.
#[no_mangle]
pub unsafe extern "C" fn mwalib_metafits_metadata_free(
    metafits_metadata_ptr: *mut MetafitsMetadata,
) -> i32 {
    // If the pointer is null, just return
    if metafits_metadata_ptr.is_null() {
        return MWALIB_SUCCESS;
    }

    let m = metafits_metadata_ptr;

    //
    // Free members first
    //
    // antennas
    antenna::ffi::Antenna::destroy_array((*m).antennas, (*m).num_ants);

    // baselines
    baseline::ffi::Baseline::destroy_array((*m).baselines, (*m).num_baselines);

    // beams
    beam::ffi::Beam::destroy_array(
        (*metafits_metadata_ptr).metafits_beams,
        (*metafits_metadata_ptr).num_metafits_beams,
    );

    // calibration fits
    calibration_fit::ffi::CalibrationFit::destroy_array(
        (*metafits_metadata_ptr).calibration_fits,
        (*metafits_metadata_ptr).num_calibration_fits,
    );

    // coarse_channels
    coarse_channel::ffi::CoarseChannel::destroy_array(
        (*metafits_metadata_ptr).metafits_coarse_chans,
        (*metafits_metadata_ptr).num_metafits_coarse_chans,
    );

    // rf inputs
    rfinput::ffi::Rfinput::destroy_array((*m).rf_inputs, (*m).num_rf_inputs);

    // signal chain corrections
    signal_chain_correction::ffi::SignalChainCorrection::destroy_array(
        (*metafits_metadata_ptr).signal_chain_corrections,
        (*metafits_metadata_ptr).num_signal_chain_corrections,
    );

    // timesteps
    timestep::ffi::TimeStep::destroy_array(
        (*metafits_metadata_ptr).metafits_timesteps,
        (*metafits_metadata_ptr).num_metafits_timesteps,
    );

    //
    // Free top level primitive arrays
    //
    // delays
    ffi_free_rust_struct(
        (*metafits_metadata_ptr).delays,
        (*metafits_metadata_ptr).num_delays,
    );

    // fine channel freqs
    ffi_free_rust_struct(
        (*metafits_metadata_ptr).metafits_fine_chan_freqs_hz,
        (*metafits_metadata_ptr).num_metafits_fine_chan_freqs_hz,
    );

    // receivers
    ffi_free_rust_struct(
        (*metafits_metadata_ptr).receivers,
        (*metafits_metadata_ptr).num_receivers,
    );

    //
    // Free top level string fields
    //
    ffi_free_rust_c_string(&mut (*metafits_metadata_ptr).hour_angle_string);
    ffi_free_rust_c_string(&mut (*metafits_metadata_ptr).grid_name);
    ffi_free_rust_c_string(&mut (*metafits_metadata_ptr).creator);
    ffi_free_rust_c_string(&mut (*metafits_metadata_ptr).project_id);
    ffi_free_rust_c_string(&mut (*metafits_metadata_ptr).obs_name);
    ffi_free_rust_c_string(&mut (*metafits_metadata_ptr).calibrator_source);
    ffi_free_rust_c_string(&mut (*metafits_metadata_ptr).metafits_filename);
    ffi_free_rust_c_string(&mut (*metafits_metadata_ptr).deripple_param);
    ffi_free_rust_c_string(&mut (*metafits_metadata_ptr).best_cal_code_ver);
    ffi_free_rust_c_string(&mut (*metafits_metadata_ptr).best_cal_fit_timestamp);
    ffi_free_rust_c_string(&mut (*metafits_metadata_ptr).best_cal_creator);

    // Free main metadata struct
    drop(Box::from_raw(metafits_metadata_ptr));

    // Return success
    MWALIB_SUCCESS
}

/// Create and return a pointer to an `MetafitsContext` struct given only a metafits file and MWAVersion.
///
/// # Arguments
///
/// * `metafits_filename` - pointer to char* buffer containing the full path and filename of a metafits file.
///
/// * `mwa_version` - enum providing mwalib with the intended mwa version which the metafits should be interpreted.
///
/// * `out_metafits_context_ptr` - A Rust-owned populated `MetafitsContext` pointer. Free with `mwalib_metafits_context_free'.
///
/// * `error_message` - pointer to already allocated buffer for any error messages to be returned to the caller.
///
/// * `error_message_length` - length of error_message char* buffer.
///
///
/// # Returns
///
/// * return MWALIB_SUCCESS on success, non-zero on failure
///
///
/// # Safety
/// * `error_message` *must* point to an already allocated `char*` buffer for any error messages.
/// * Caller *must* call the `mwalib_metafits_context_free` function to release the rust memory.
#[no_mangle]
pub unsafe extern "C" fn mwalib_metafits_context_new(
    metafits_filename: *const c_char,
    mwa_version: MWAVersion,
    out_metafits_context_ptr: &mut *mut MetafitsContext,
    error_message: *mut c_char,
    error_message_length: size_t,
) -> i32 {
    let m = match CStr::from_ptr(metafits_filename).to_str() {
        Ok(s) => s,
        Err(_) => {
            set_c_string(
                "invalid UTF-8 in metafits_filename",
                error_message as *mut c_char,
                error_message_length,
            );
            return MWALIB_FAILURE;
        }
    };

    let context = match MetafitsContext::new(m, Some(mwa_version)) {
        Ok(c) => c,
        Err(e) => {
            set_c_string(
                &format!("{}", e),
                error_message as *mut c_char,
                error_message_length,
            );
            // Return failure
            return MWALIB_FAILURE;
        }
    };

    *out_metafits_context_ptr = Box::into_raw(Box::new(context));

    // Return success
    MWALIB_SUCCESS
}

/// Create and return a pointer to an `MetafitsContext` struct given only a metafits file. Same as mwalib_metafits_context_new, but mwalib will guess the MWAVersion.
///
/// # Arguments
///
/// * `metafits_filename` - pointer to char* buffer containing the full path and filename of a metafits file.
///
/// * `out_metafits_context_ptr` - A Rust-owned populated `MetafitsContext` pointer. Free with `mwalib_metafits_context_free'.
///
/// * `error_message` - pointer to already allocated buffer for any error messages to be returned to the caller.
///
/// * `error_message_length` - length of error_message char* buffer.
///
///
/// # Returns
///
/// * return MWALIB_SUCCESS on success, non-zero on failure
///
///
/// # Safety
/// * `error_message` *must* point to an already allocated `char*` buffer for any error messages.
/// * Caller *must* call the `mwalib_metafits_context_free` function to release the rust memory.
#[no_mangle]
pub unsafe extern "C" fn mwalib_metafits_context_new2(
    metafits_filename: *const c_char,
    out_metafits_context_ptr: &mut *mut MetafitsContext,
    error_message: *mut c_char,
    error_message_length: size_t,
) -> i32 {
    let m = match CStr::from_ptr(metafits_filename).to_str() {
        Ok(s) => s,
        Err(_) => {
            set_c_string(
                "invalid UTF-8 in metafits_filename",
                error_message as *mut c_char,
                error_message_length,
            );
            return MWALIB_FAILURE;
        }
    };

    let context = match MetafitsContext::new(m, None) {
        Ok(c) => c,
        Err(e) => {
            set_c_string(
                &format!("{}", e),
                error_message as *mut c_char,
                error_message_length,
            );
            // Return failure
            return MWALIB_FAILURE;
        }
    };

    *out_metafits_context_ptr = Box::into_raw(Box::new(context));

    // Return success
    MWALIB_SUCCESS
}

/// Generates an expected filename, given a MetafitsContext, timestep index and coarse channel index.
///
///
/// # Arguments
///
/// * `metafits_context_ptr` - pointer to an already populated `MetafitsContext` object
///
/// * `metafits_timestep_index` - the timestep index you are generating this filename for.
///
/// * `metafits_coarse_chan_index` - the coarse_chan index you are generating this filename for.
///
/// * `out_filename_ptr` - Pointer to a char* buffer which has already been allocated, for storing the filename.
///
/// * `out_filename_len` - Length of char* buffer allocated by caller in C.
///
/// * `error_message` - pointer to already allocated buffer for any error messages to be returned to the caller.
///
/// * `error_message_length` - length of error_message char* buffer.
///
///
/// # Returns
///
/// * MWALIB_SUCCESS on success, non-zero on failure
///
///
/// # Safety
/// * `out_filename_ptr` *must* point to an already allocated char* buffer for the output filename to be written to.
/// * `error_message` *must* point to an already allocated char* buffer for any error messages.
/// * `metafits_context_ptr` must contain an MetafitsContext object already populated via `mwalib_metafits_context_new`
///   It is up to the caller to:
///   - Free `out_filename_ptr` once finished with the buffer.
///   - Free `error_message` once finished with the buffer.
#[no_mangle]
pub unsafe extern "C" fn mwalib_metafits_get_expected_volt_filename(
    metafits_context_ptr: *const MetafitsContext,
    metafits_timestep_index: usize,
    metafits_coarse_chan_index: usize,
    out_filename_ptr: *const c_char,
    out_filename_len: size_t,
    error_message: *mut c_char,
    error_message_length: size_t,
) -> i32 {
    if metafits_context_ptr.is_null() {
        set_c_string(
            "mwalib_metafits_get_expected_voltage_filename() ERROR: null pointer for metafits_context_ptr passed in",
            error_message as *mut c_char,
            error_message_length,
        );
        return MWALIB_FAILURE;
    }

    let context = &*metafits_context_ptr;

    match context
        .generate_expected_volt_filename(metafits_timestep_index, metafits_coarse_chan_index)
    {
        Err(e) => {
            set_c_string(
                &e.to_string(),
                error_message as *mut c_char,
                error_message_length,
            );
            MWALIB_FAILURE
        }
        Ok(s) => {
            set_c_string(&s, out_filename_ptr as *mut c_char, out_filename_len);

            // Return success
            MWALIB_SUCCESS
        }
    }
}

/// Display an `MetafitsContext` struct.
///
///
/// # Arguments
///
/// * `metafits_context_ptr` - pointer to an already populated `MetafitsContext` object
///
/// * `error_message` - pointer to already allocated buffer for any error messages to be returned to the caller.
///
/// * `error_message_length` - length of error_message char* buffer.
///
///
/// # Returns
///
/// * MWALIB_SUCCESS on success, non-zero on failure
///
///
/// # Safety
/// * `error_message` *must* point to an already allocated char* buffer for any error messages.
/// * `metafits_context_ptr` must contain an MetafitsContext object already populated via `mwalib_metafits_context_new`
#[no_mangle]
pub unsafe extern "C" fn mwalib_metafits_context_display(
    metafits_context_ptr: *const MetafitsContext,
    error_message: *mut c_char,
    error_message_length: size_t,
) -> i32 {
    if metafits_context_ptr.is_null() {
        set_c_string(
            "mwalib_metafits_context_display() ERROR: null pointer for metafits_context_ptr passed in",
            error_message as *mut c_char,
            error_message_length,
        );
        return MWALIB_FAILURE;
    }

    let context = &*metafits_context_ptr;

    println!("{}", context);

    // Return success
    MWALIB_SUCCESS
}

/// Free a previously-allocated `MetafitsContext` struct (and it's members).
///
/// # Arguments
///
/// * `metafits_context_ptr` - pointer to an already populated `MetafitsContext` object
///
///
/// # Returns
///
/// * MWALIB_SUCCESS on success, non-zero on failure
///
///
/// # Safety
/// * This must be called once caller is finished with the `MetafitsContext` object
/// * `metafits_context_ptr` must point to a populated `MetafitsContext` object from the `mwalib_metafits_context_new` functions.
/// * `metafits_context_ptr` must not have already been freed.
#[no_mangle]
#[allow(unused_must_use)]
pub unsafe extern "C" fn mwalib_metafits_context_free(
    metafits_context_ptr: *mut MetafitsContext,
) -> i32 {
    if metafits_context_ptr.is_null() {
        return MWALIB_SUCCESS;
    }

    // Release correlator context if applicable
    drop(Box::from_raw(metafits_context_ptr));

    // Return success
    MWALIB_SUCCESS
}
