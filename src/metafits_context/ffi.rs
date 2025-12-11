// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::{
    antenna, baseline, beam, calibration_fit, coarse_channel,
    ffi::{
        ffi_array_to_boxed_slice, ffi_free_c_boxed_slice, free_c_string, rust_string_to_buf,
        set_c_string, MWALIB_FAILURE, MWALIB_SUCCESS,
    },
    rfinput, signal_chain_correction, timestep,
    types::DataFileType,
    CableDelaysApplied, CorrelatorContext, GeometricDelaysApplied, MWAMode, MWAVersion,
    MetafitsContext, VoltageContext, MAX_RECEIVER_CHANNELS,
};
use libc::size_t;
use std::{
    ffi::{c_char, CStr, CString},
    slice,
};

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

    // Populate baselines
    let mut baseline_vec: Vec<baseline::ffi::Baseline> = Vec::new();
    for item in metafits_context.baselines.iter() {
        let out_item = {
            let baseline::Baseline {
                ant1_index,
                ant2_index,
            } = item;
            baseline::ffi::Baseline {
                ant1_index: *ant1_index,
                ant2_index: *ant2_index,
            }
        };

        baseline_vec.push(out_item);
    }

    // Populate antennas
    let mut antenna_vec: Vec<antenna::ffi::Antenna> = Vec::new();
    for item in metafits_context.antennas.iter() {
        let out_item = {
            let antenna::Antenna {
                ant,
                tile_id,
                tile_name: _,
                rfinput_x,
                rfinput_y,
                electrical_length_m,
                north_m,
                east_m,
                height_m,
            } = item;

            let tile_name_c_str = rust_string_to_buf(item.tile_name.clone());

            antenna::ffi::Antenna {
                ant: *ant,
                tile_id: *tile_id,
                tile_name: tile_name_c_str,
                rfinput_x: metafits_context
                    .rf_inputs
                    .iter()
                    .position(|x| x == rfinput_x)
                    .unwrap(),
                rfinput_y: metafits_context
                    .rf_inputs
                    .iter()
                    .position(|y| y == rfinput_y)
                    .unwrap(),
                electrical_length_m: *electrical_length_m,
                north_m: *north_m,
                east_m: *east_m,
                height_m: *height_m,
            }
        };

        antenna_vec.push(out_item);
    }

    // Populate rf_inputs
    let mut rfinput_vec: Vec<rfinput::ffi::Rfinput> = Vec::new();
    for item in metafits_context.rf_inputs.iter() {
        let out_item = {
            let rfinput::Rfinput {
                input,
                ant,
                tile_id,
                tile_name,
                pol,
                electrical_length_m,
                north_m,
                east_m,
                height_m,
                vcs_order,
                subfile_order,
                flagged,
                digital_gains,
                dipole_gains,
                dipole_delays,
                rec_number,
                rec_slot_number,
                rec_type,
                flavour,
                has_whitening_filter,
                calib_delay,
                calib_gains,
                signal_chain_corrections_index,
            } = item;

            let calib_delay = calib_delay.unwrap_or(f32::NAN);
            let calib_gains_vec: Vec<f32> = calib_gains
                .clone()
                .unwrap_or(vec![f32::NAN; metafits_context.num_metafits_coarse_chans]);
            let num_calib_gains = calib_gains_vec.len();

            rfinput::ffi::Rfinput {
                input: *input,
                ant: *ant,
                tile_id: *tile_id,
                tile_name: CString::new(tile_name.replace('\0', ""))
                    .unwrap_or_else(|_| CString::new("").unwrap())
                    .into_raw(),
                pol: CString::new(pol.to_string()).unwrap().into_raw(),
                electrical_length_m: *electrical_length_m,
                north_m: *north_m,
                east_m: *east_m,
                height_m: *height_m,
                vcs_order: *vcs_order,
                subfile_order: *subfile_order,
                flagged: *flagged,
                digital_gains: ffi_array_to_boxed_slice(digital_gains.clone()),
                num_digital_gains: digital_gains.len(),
                dipole_gains: ffi_array_to_boxed_slice(dipole_gains.clone()),
                num_dipole_gains: dipole_gains.len(),
                dipole_delays: ffi_array_to_boxed_slice(dipole_delays.clone()),
                num_dipole_delays: dipole_delays.len(),
                rec_number: *rec_number,
                rec_slot_number: *rec_slot_number,
                rec_type: *rec_type,
                flavour: CString::new(flavour.replace('\0', ""))
                    .unwrap_or_else(|_| CString::new("").unwrap())
                    .into_raw(),
                has_whitening_filter: *has_whitening_filter,
                calib_delay,
                calib_gains: ffi_array_to_boxed_slice(calib_gains_vec),
                num_calib_gains,
                signal_chain_corrections_index: signal_chain_corrections_index
                    .unwrap_or(MAX_RECEIVER_CHANNELS),
            }
        };
        rfinput_vec.push(out_item);
    }

    // Populate metafits coarse channels
    let mut coarse_chan_vec: Vec<coarse_channel::ffi::CoarseChannel> = Vec::new();

    for item in metafits_context.metafits_coarse_chans.iter() {
        let out_item = {
            let coarse_channel::CoarseChannel {
                corr_chan_number,
                rec_chan_number,
                gpubox_number,
                chan_width_hz,
                chan_start_hz,
                chan_centre_hz,
                chan_end_hz,
            } = item;
            coarse_channel::ffi::CoarseChannel {
                corr_chan_number: *corr_chan_number,
                rec_chan_number: *rec_chan_number,
                gpubox_number: *gpubox_number,
                chan_width_hz: *chan_width_hz,
                chan_start_hz: *chan_start_hz,
                chan_centre_hz: *chan_centre_hz,
                chan_end_hz: *chan_end_hz,
            }
        };

        coarse_chan_vec.push(out_item);
    }

    // Populate metafits timesteps
    let mut timestep_vec: Vec<timestep::ffi::TimeStep> = Vec::new();

    for item in metafits_context.metafits_timesteps.iter() {
        let out_item = {
            let timestep::TimeStep {
                unix_time_ms,
                gps_time_ms,
            } = item;
            timestep::ffi::TimeStep {
                unix_time_ms: *unix_time_ms,
                gps_time_ms: *gps_time_ms,
            }
        };
        timestep_vec.push(out_item);
    }

    // Populate signal chain corrections
    let mut signal_chain_corrections_vec: Vec<signal_chain_correction::ffi::SignalChainCorrection> =
        Vec::new();

    if let Some(v) = &metafits_context.signal_chain_corrections {
        for item in v.iter() {
            let out_item = {
                let signal_chain_correction::SignalChainCorrection {
                    receiver_type,
                    whitening_filter,
                    corrections,
                } = item;
                signal_chain_correction::ffi::SignalChainCorrection {
                    receiver_type: *receiver_type,
                    whitening_filter: *whitening_filter,
                    corrections: ffi_array_to_boxed_slice(corrections.clone()),
                }
            };
            signal_chain_corrections_vec.push(out_item);
        }
    }

    // Populate calibration fits
    let mut calibration_fits_vec: Vec<calibration_fit::ffi::CalibrationFit> = Vec::new();

    if let Some(v) = &metafits_context.calibration_fits {
        for item in v.iter() {
            let out_item = {
                let calibration_fit::CalibrationFit {
                    rf_input: _,
                    delay_metres,
                    intercept_metres,
                    gains,
                    gain_polynomial_fit0,
                    gain_polynomial_fit1,
                    phase_fit_quality,
                    gain_fit_quality,
                } = item;

                calibration_fit::ffi::CalibrationFit {
                    rf_input: metafits_context
                        .rf_inputs
                        .iter()
                        .position(|x| x.ant == item.rf_input.ant && x.pol == item.rf_input.pol)
                        .unwrap(),
                    delay_metres: *delay_metres,
                    intercept_metres: *intercept_metres,
                    gains: ffi_array_to_boxed_slice(gains.clone()),
                    gain_polynomial_fit0: ffi_array_to_boxed_slice(gain_polynomial_fit0.clone()),
                    gain_polynomial_fit1: ffi_array_to_boxed_slice(gain_polynomial_fit1.clone()),
                    phase_fit_quality: *phase_fit_quality,
                    gain_fit_quality: *gain_fit_quality,
                }
            };
            calibration_fits_vec.push(out_item);
        }
    }

    // Populate beams
    let mut beams_vec: Vec<beam::ffi::Beam> = Vec::new();
    if let Some(v) = &metafits_context.metafits_beams {
        for item in v.iter() {
            // Get a list of antenna indicies for this beam
            let tileset_antenna_indices = item
                .antennas
                .iter()
                .filter_map(|sel| metafits_context.antennas.iter().position(|a| a == sel))
                .collect();

            // Get a list of coarse channel indices for this beam
            let coarse_channel_indices = item
                .coarse_channels
                .iter()
                .filter_map(|sel| {
                    metafits_context
                        .metafits_coarse_chans
                        .iter()
                        .position(|c| c == sel)
                })
                .collect();

            let out_item = {
                let beam::Beam {
                    number,
                    coherent,
                    az_deg,
                    alt_deg,
                    ra_deg,
                    dec_deg,
                    tle,
                    num_time_samples_to_average,
                    frequency_resolution_hz,
                    coarse_channels: _,
                    num_coarse_chans,
                    antennas: _,
                    num_ants: num_antennas,
                    polarisation,
                    data_file_type,
                    creator,
                    modtime,
                    beam_index,
                } = item;
                beam::ffi::Beam {
                    number: *number,
                    coherent: *coherent,
                    az_deg: az_deg.unwrap_or_default(),
                    alt_deg: alt_deg.unwrap_or_default(),
                    ra_deg: ra_deg.unwrap_or_default(),
                    dec_deg: dec_deg.unwrap_or_default(),
                    tle: CString::new(tle.clone().unwrap_or_default().replace('\0', ""))
                        .unwrap_or_else(|_| CString::new("").unwrap())
                        .into_raw(),
                    num_time_samples_to_average: *num_time_samples_to_average,
                    frequency_resolution_hz: *frequency_resolution_hz,
                    coarse_channels: ffi_array_to_boxed_slice(coarse_channel_indices),
                    num_coarse_chans: *num_coarse_chans,
                    antennas: ffi_array_to_boxed_slice(tileset_antenna_indices),
                    num_ants: *num_antennas,
                    polarisation: CString::new(
                        polarisation.clone().unwrap_or_default().replace('\0', ""),
                    )
                    .unwrap_or_else(|_| CString::new("").unwrap())
                    .into_raw(),
                    data_file_type: *data_file_type as DataFileType,
                    creator: CString::new(creator.replace('\0', ""))
                        .unwrap_or_else(|_| CString::new("").unwrap())
                        .into_raw(),
                    modtime: chrono::DateTime::<chrono::Utc>::from(*modtime).timestamp(),
                    beam_index: match beam_index {
                        Some(idx) => *idx as i32,
                        None => -1,
                    },
                }
            };
            beams_vec.push(out_item);
        }
    }

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
            num_ants,
            antennas: _, // This is populated seperately
            num_rf_inputs,
            rf_inputs: _, // This is populated seperately
            num_ant_pols,
            num_metafits_timesteps,
            metafits_timesteps: _, // This is populated seperately
            num_metafits_coarse_chans,
            metafits_coarse_chans: _, // This is populated seperately
            num_metafits_fine_chan_freqs,
            metafits_fine_chan_freqs_hz,
            obs_bandwidth_hz,
            coarse_chan_width_hz,
            centre_freq_hz,
            num_baselines,
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
            num_signal_chain_corrections,
            calibration_fits: _, // This is populated seperately
            num_calibration_fits,
            metafits_beams: _, // This is populated seperately
            num_metafits_beams: num_beams,
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
            hour_angle_string: CString::new(hour_angle_string.replace('\0', ""))
                .unwrap_or_else(|_| CString::new("").unwrap())
                .into_raw(),
            grid_name: CString::new(grid_name.replace('\0', ""))
                .unwrap_or_else(|_| CString::new("").unwrap())
                .into_raw(),
            grid_number: *grid_number,
            creator: CString::new(creator.replace('\0', ""))
                .unwrap_or_else(|_| CString::new("").unwrap())
                .into_raw(),
            project_id: CString::new(project_id.replace('\0', ""))
                .unwrap_or_else(|_| CString::new("").unwrap())
                .into_raw(),
            obs_name: CString::new(obs_name.replace('\0', ""))
                .unwrap_or_else(|_| CString::new("").unwrap())
                .into_raw(),
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
            receivers: ffi_array_to_boxed_slice(receivers.clone()),
            num_receivers: *num_receivers,
            delays: ffi_array_to_boxed_slice(delays.clone()),
            num_delays: *num_delays,
            calibrator: *calibrator,
            calibrator_source: CString::new(calibrator_source.replace('\0', ""))
                .unwrap_or_else(|_| CString::new("").unwrap())
                .into_raw(),
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
            num_ants: *num_ants,
            antennas: ffi_array_to_boxed_slice(antenna_vec),
            num_rf_inputs: *num_rf_inputs,
            rf_inputs: ffi_array_to_boxed_slice(rfinput_vec),
            num_ant_pols: *num_ant_pols,
            num_baselines: *num_baselines,
            baselines: ffi_array_to_boxed_slice(baseline_vec),
            num_visibility_pols: *num_visibility_pols,
            num_metafits_coarse_chans: *num_metafits_coarse_chans,
            metafits_coarse_chans: ffi_array_to_boxed_slice(coarse_chan_vec),
            num_metafits_fine_chan_freqs_hz: *num_metafits_fine_chan_freqs,
            metafits_fine_chan_freqs_hz: ffi_array_to_boxed_slice(
                metafits_fine_chan_freqs_hz.clone(),
            ),
            num_metafits_timesteps: *num_metafits_timesteps,
            metafits_timesteps: ffi_array_to_boxed_slice(timestep_vec),
            obs_bandwidth_hz: *obs_bandwidth_hz,
            coarse_chan_width_hz: *coarse_chan_width_hz,
            centre_freq_hz: *centre_freq_hz,
            metafits_filename: CString::new(metafits_filename.replace('\0', ""))
                .unwrap_or_else(|_| CString::new("").unwrap())
                .into_raw(),
            oversampled: *oversampled,
            deripple_applied: *deripple_applied,
            deripple_param: CString::new(deripple_param.replace('\0', ""))
                .unwrap_or_else(|_| CString::new("").unwrap())
                .into_raw(),
            best_cal_fit_id: best_cal_fit_id.unwrap_or_else(|| 0),
            best_cal_obs_id: best_cal_obs_id.unwrap_or_else(|| 0),
            best_cal_code_ver: CString::new(
                best_cal_code_ver
                    .clone()
                    .unwrap_or_default()
                    .replace('\0', ""),
            )
            .unwrap_or_else(|_| CString::new("").unwrap())
            .into_raw(),
            best_cal_fit_timestamp: CString::new(
                best_cal_fit_timestamp
                    .clone()
                    .unwrap_or_default()
                    .replace('\0', ""),
            )
            .unwrap_or_else(|_| CString::new("").unwrap())
            .into_raw(),
            best_cal_creator: CString::new(
                best_cal_creator
                    .clone()
                    .unwrap_or_default()
                    .replace('\0', ""),
            )
            .unwrap_or_else(|_| CString::new("").unwrap())
            .into_raw(),
            best_cal_fit_iters: best_cal_fit_iters.unwrap_or_else(|| 0),
            best_cal_fit_iter_limit: best_cal_fit_iter_limit.unwrap_or_else(|| 0),
            signal_chain_corrections: ffi_array_to_boxed_slice(signal_chain_corrections_vec),
            num_signal_chain_corrections: *num_signal_chain_corrections,
            calibration_fits: ffi_array_to_boxed_slice(calibration_fits_vec),
            num_calibration_fits: *num_calibration_fits,
            metafits_beams: ffi_array_to_boxed_slice(beams_vec),
            num_metafits_beams: *num_beams,
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

    //
    // Free members first
    //
    // baselines
    ffi_free_c_boxed_slice(
        (*metafits_metadata_ptr).baselines,
        (*metafits_metadata_ptr).num_baselines,
    );

    // antennas
    if !(*metafits_metadata_ptr).antennas.is_null() {
        // Reconstruct a fat slice first then box from that raw slice to allow Rust to deallocate the memory
        let slice: &mut [antenna::ffi::Antenna] = slice::from_raw_parts_mut(
            (*metafits_metadata_ptr).antennas,
            (*metafits_metadata_ptr).num_ants,
        );
        // Now for each item we need to free anything on the heap
        for i in slice.iter_mut() {
            //drop(CString::from_raw(i.tile_name));
            free_c_string(i.tile_name);
        }

        // Free the memory for the slice
        drop(Box::from_raw(slice));
    }

    // rf inputs
    if !(*metafits_metadata_ptr).rf_inputs.is_null() {
        // Reconstruct a fat slice first then box from that raw slice to allow Rust to deallocate the memory
        let slice: &mut [rfinput::ffi::Rfinput] = slice::from_raw_parts_mut(
            (*metafits_metadata_ptr).rf_inputs,
            (*metafits_metadata_ptr).num_rf_inputs,
        );
        // Now for each item we need to free anything on the heap
        for i in slice.iter_mut() {
            drop(CString::from_raw(i.tile_name));
            drop(CString::from_raw(i.pol));

            ffi_free_c_boxed_slice(i.digital_gains, i.num_digital_gains);
            ffi_free_c_boxed_slice(i.dipole_gains, i.num_dipole_gains);
            ffi_free_c_boxed_slice(i.dipole_delays, i.num_dipole_delays);
            ffi_free_c_boxed_slice(i.calib_gains, i.num_calib_gains);

            drop(CString::from_raw(i.flavour));
        }

        // Free the memory for the slice
        drop(Box::from_raw(slice));
    }

    // coarse_channels
    ffi_free_c_boxed_slice(
        (*metafits_metadata_ptr).metafits_coarse_chans,
        (*metafits_metadata_ptr).num_metafits_coarse_chans,
    );

    // timesteps
    ffi_free_c_boxed_slice(
        (*metafits_metadata_ptr).metafits_timesteps,
        (*metafits_metadata_ptr).num_metafits_timesteps,
    );

    // receivers
    ffi_free_c_boxed_slice(
        (*metafits_metadata_ptr).receivers,
        (*metafits_metadata_ptr).num_receivers,
    );

    // delays
    ffi_free_c_boxed_slice(
        (*metafits_metadata_ptr).delays,
        (*metafits_metadata_ptr).num_delays,
    );

    // fine channel freqs
    ffi_free_c_boxed_slice(
        (*metafits_metadata_ptr).metafits_fine_chan_freqs_hz,
        (*metafits_metadata_ptr).num_metafits_fine_chan_freqs_hz,
    );

    // signal chain corrections
    if !(*metafits_metadata_ptr).signal_chain_corrections.is_null() {
        // Reconstruct a fat slice first then box from that raw slice to allow Rust to deallocate the memory
        let slice: &mut [signal_chain_correction::ffi::SignalChainCorrection] =
            slice::from_raw_parts_mut(
                (*metafits_metadata_ptr).signal_chain_corrections,
                (*metafits_metadata_ptr).num_signal_chain_corrections,
            );

        // Now for each item we need to free anything on the heap
        for i in slice.iter_mut() {
            if !i.corrections.is_null() {
                drop(Box::from_raw(i.corrections));
            }
        }

        drop(Box::from_raw(
            (*metafits_metadata_ptr).signal_chain_corrections,
        ));
    }

    // calibration fits
    if !(*metafits_metadata_ptr).calibration_fits.is_null() {
        // Reconstruct a fat slice first then box from that raw slice to allow Rust to deallocate the memory
        let slice: &mut [calibration_fit::ffi::CalibrationFit] = slice::from_raw_parts_mut(
            (*metafits_metadata_ptr).calibration_fits,
            (*metafits_metadata_ptr).num_calibration_fits,
        );

        // Now for each item we need to free anything on the heap
        for i in slice.iter_mut() {
            if !i.gains.is_null() {
                drop(Box::from_raw(i.gains));
            }

            if !i.gains.is_null() {
                drop(Box::from_raw(i.gain_polynomial_fit0));
            }

            if !i.gains.is_null() {
                drop(Box::from_raw(i.gain_polynomial_fit1));
            }
        }

        drop(Box::from_raw((*metafits_metadata_ptr).calibration_fits));
    }

    // beams
    if !(*metafits_metadata_ptr).metafits_beams.is_null() {
        // Reconstruct a fat slice first then box from that raw slice to allow Rust to deallocate the memory
        let slice: &mut [beam::ffi::Beam] = slice::from_raw_parts_mut(
            (*metafits_metadata_ptr).metafits_beams,
            (*metafits_metadata_ptr).num_metafits_beams,
        );

        // Now for each item we need to free anything on the heap
        for i in slice.iter_mut() {
            if !i.antennas.is_null() {
                drop(Box::from_raw(i.antennas));
            }

            if !i.coarse_channels.is_null() {
                drop(Box::from_raw(i.coarse_channels));
            }

            drop(CString::from_raw(i.creator));
            drop(CString::from_raw(i.tle));
            drop(CString::from_raw(i.polarisation));
        }

        drop(Box::from_raw((*metafits_metadata_ptr).metafits_beams));
    }

    // Free top level string fields
    if !(*metafits_metadata_ptr).hour_angle_string.is_null() {
        drop(CString::from_raw(
            (*metafits_metadata_ptr).hour_angle_string,
        ));
    }

    if !(*metafits_metadata_ptr).grid_name.is_null() {
        drop(CString::from_raw((*metafits_metadata_ptr).grid_name));
    }

    if !(*metafits_metadata_ptr).creator.is_null() {
        drop(CString::from_raw((*metafits_metadata_ptr).creator));
    }

    if !(*metafits_metadata_ptr).project_id.is_null() {
        drop(CString::from_raw((*metafits_metadata_ptr).project_id));
    }

    if !(*metafits_metadata_ptr).calibrator_source.is_null() {
        drop(CString::from_raw(
            (*metafits_metadata_ptr).calibrator_source,
        ));
    }

    if !(*metafits_metadata_ptr).metafits_filename.is_null() {
        drop(CString::from_raw(
            (*metafits_metadata_ptr).metafits_filename,
        ));
    }

    if !(*metafits_metadata_ptr).deripple_param.is_null() {
        drop(CString::from_raw((*metafits_metadata_ptr).deripple_param));
    }

    if !(*metafits_metadata_ptr).best_cal_code_ver.is_null() {
        drop(CString::from_raw(
            (*metafits_metadata_ptr).best_cal_code_ver,
        ));
    }

    if !(*metafits_metadata_ptr).best_cal_fit_timestamp.is_null() {
        drop(CString::from_raw(
            (*metafits_metadata_ptr).best_cal_fit_timestamp,
        ));
    }

    if !(*metafits_metadata_ptr).best_cal_creator.is_null() {
        drop(CString::from_raw((*metafits_metadata_ptr).best_cal_creator));
    }

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
