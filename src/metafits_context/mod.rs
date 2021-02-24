// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

/*!
The main interface to MWA data.
 */
use std::f64::consts::FRAC_PI_2;
use std::fmt;

use chrono::{DateTime, Duration, FixedOffset};

use crate::antenna::*;
use crate::baseline::*;
use crate::rfinput::*;
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

/// `mwalib` metafits context. This represents the basic metadata for the observation.
///
#[derive(Clone, Debug)]
pub struct MetafitsContext {
    /// Observation id
    pub obsid: u32,
    /// Latitude of centre point of MWA in raidans
    pub mwa_latitude_radians: f64,
    /// Longitude of centre point of MWA in raidans
    pub mwa_longitude_radians: f64,
    /// Altitude of centre poing of MWA in metres
    pub mwa_altitude_metres: f64,
    /// the velocity factor of electic fields in RG-6 like coax
    pub coax_v_factor: f64,
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
    /// AZIMUTH of the pointing centre in degrees
    pub azimuth_degrees: f64,
    /// ALTITUDE (a.k.a. elevation) of the pointing centre in degrees
    pub altitude_degrees: f64,
    /// Zenith angle of the pointing centre in degrees
    pub zenith_angle_degrees: f64,
    /// AZIMUTH of the pointing centre in radians
    pub azimuth_radians: f64,
    /// ALTITUDE (a.k.a. elevation) of the pointing centre in radians
    pub altitude_radians: f64,
    /// Zenith angle of the pointing centre in radians
    pub zenith_angle_radians: f64,
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
    /// Local Sidereal Time in radians
    pub lst_radians: f64,
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
    /// Correlator fine_channel_resolution
    pub correlator_fine_channel_width_hz: u32,
    /// Correlator mode dump time
    pub correlator_integration_time_milliseconds: u64,
    /// Number of fine channels in each coarse channel
    pub num_fine_channels_per_coarse: usize,
    /// RECVRS    // Array of receiver numbers (this tells us how many receivers too)
    pub receivers: Vec<usize>,
    /// DELAYS    // Array of delays
    pub delays: Vec<u32>,
    /// ATTEN_DB  // global analogue attenuation, in dB
    pub global_analogue_attenuation_db: f64,
    /// Seconds of bad data after observation starts
    pub quack_time_duration_milliseconds: u64,
    /// OBSID+QUACKTIM as Unix timestamp (first good timestep)
    pub good_time_unix_milliseconds: u64,
    /// Total number of antennas (tiles) in the array
    pub num_antennas: usize,
    /// We also have just the antennas
    pub antennas: Vec<Antenna>,
    /// Total number of rf_inputs (tiles * 2 pols X&Y)    
    pub num_rf_inputs: usize,
    /// The Metafits defines an rf chain for antennas(tiles) * pol(X,Y)
    pub rf_inputs: Vec<RFInput>,
    /// Number of antenna pols. e.g. X and Y
    pub num_antenna_pols: usize,
    /// Number of coarse channels we should have
    pub num_coarse_channels: usize,
    /// Total bandwidth of observation assuming we have all coarse channels
    pub observation_bandwidth_hz: u32,
    /// Bandwidth of each coarse channel
    pub coarse_channel_width_hz: u32,
    /// The value of the FREQCENT key in the metafits file, but in Hz.
    pub metafits_centre_freq_hz: u32,
    /// Number of baselines stored. This is autos plus cross correlations
    pub num_baselines: usize,
    /// Baslines
    pub baselines: Vec<Baseline>,
    /// Number of polarisation combinations in the visibilities e.g. XX,XY,YX,YY == 4
    pub num_visibility_pols: usize,
    /// Visibility polarisations
    pub visibility_pols: Vec<VisibilityPol>,
    /// Filename of the metafits we were given
    pub metafits_filename: String,
}

impl MetafitsContext {
    pub fn new<T: AsRef<std::path::Path>>(metafits: &T) -> Result<Self, MwalibError> {
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
        let quack_time_duration_milliseconds: u64 = {
            let qt: f64 = get_required_fits_key!(&mut metafits_fptr, &metafits_hdu, "QUACKTIM")?;
            (qt * 1000.).round() as _
        };
        let good_time_unix_milliseconds: u64 = {
            let gt: f64 = get_required_fits_key!(&mut metafits_fptr, &metafits_hdu, "GOODTIME")?;
            (gt * 1000.).round() as _
        };

        // Create a vector of rf_input structs from the metafits
        let num_rf_inputs: usize =
            get_required_fits_key!(&mut metafits_fptr, &metafits_hdu, "NINPUTS")?;

        // There are twice as many inputs as
        // there are antennas; halve that value.
        let num_antennas = num_rf_inputs / 2;

        // Create a vector of rf_input structs from the metafits
        let mut rf_inputs: Vec<RFInput> = RFInput::populate_rf_inputs(
            num_rf_inputs,
            &mut metafits_fptr,
            metafits_tile_table_hdu,
            coax_v_factor,
        )?;

        // Sort the rf_inputs back into the correct output order
        rf_inputs.sort_by_key(|k| k.subfile_order);

        // Now populate the antennas (note they need to be sorted by subfile_order)
        let antennas: Vec<Antenna> = Antenna::populate_antennas(&rf_inputs);

        // Always assume that MWA antennas have 2 pols
        let num_antenna_pols = 2;

        // Populate baselines
        let baselines = Baseline::populate_baselines(num_antennas);

        // Populate the pols that come out of the correlator
        let visibility_pols = VisibilityPol::populate_visibility_pols();
        let num_visibility_pols = visibility_pols.len();

        // `num_baselines` is the number of cross-correlations + the number of
        // auto-correlations.
        let num_baselines = (num_antennas / 2) * (num_antennas + 1);

        // The FREQCENT value in the metafits is in units of kHz - make it Hz.
        let metafits_centre_freq_hz: u32 = {
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
        let zenith_angle_degrees: f64 = FRAC_PI_2 - altitude_degrees;
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
        // We need to get the correlator integration time
        let integration_time_milliseconds: u64 = {
            let it: f64 = get_required_fits_key!(&mut metafits_fptr, &metafits_hdu, "INTTIME")?;
            (it * 1000.) as _
        };
        let receivers_string: String =
            get_required_fits_key!(&mut metafits_fptr, &metafits_hdu, "RECVRS")?;

        let receivers: Vec<usize> = receivers_string
            .replace(&['\'', '&'][..], "")
            .split(',')
            .map(|s| s.parse().unwrap())
            .collect();

        let delays_string: String =
            get_required_fits_key!(&mut metafits_fptr, &metafits_hdu, "DELAYS")?;

        let delays: Vec<u32> = delays_string
            .replace(&['\'', '&'][..], "")
            .split(',')
            .map(|s| s.parse().unwrap())
            .collect();

        let global_analogue_attenuation_db: f64 =
            get_required_fits_key!(&mut metafits_fptr, &metafits_hdu, "ATTEN_DB")?;

        // observation bandwidth (read from metafits in MHz)
        let metafits_observation_bandwidth_hz: u32 = {
            let bw: f64 = get_required_fits_key!(&mut metafits_fptr, &metafits_hdu, "BANDWDTH")?;
            (bw * 1e6).round() as _
        };

        // Populate coarse channels
        let (num_coarse_channels, coarse_channel_width_hz) =
            coarse_channel::CoarseChannel::populate_metafits_coarse_channels(
                &mut metafits_fptr,
                &metafits_hdu,
                metafits_observation_bandwidth_hz,
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

        Ok(MetafitsContext {
            obsid,
            mwa_latitude_radians: MWA_LATITUDE_RADIANS,
            mwa_longitude_radians: MWA_LONGITUDE_RADIANS,
            mwa_altitude_metres: MWA_ALTITUDE_METRES,
            coax_v_factor,
            scheduled_start_gpstime_milliseconds,
            scheduled_end_gpstime_milliseconds,
            scheduled_start_unix_time_milliseconds,
            scheduled_end_unix_time_milliseconds,
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
            zenith_angle_degrees,
            azimuth_radians: azimuth_degrees.to_radians(),
            altitude_radians: altitude_degrees.to_radians(),
            zenith_angle_radians: zenith_angle_degrees.to_radians(),
            sun_altitude_degrees,
            sun_distance_degrees,
            moon_distance_degrees,
            jupiter_distance_degrees,
            lst_degrees,
            lst_radians: lst_degrees.to_radians(),
            hour_angle_string,
            grid_name,
            grid_number,
            creator,
            project_id,
            observation_name,
            mode,
            correlator_fine_channel_width_hz: fine_channel_width_hz,
            correlator_integration_time_milliseconds: integration_time_milliseconds,
            num_fine_channels_per_coarse,
            receivers,
            delays,
            global_analogue_attenuation_db,
            quack_time_duration_milliseconds,
            good_time_unix_milliseconds,
            num_antennas,
            antennas,
            num_rf_inputs,
            rf_inputs,
            num_antenna_pols,
            num_coarse_channels,
            observation_bandwidth_hz,
            coarse_channel_width_hz,
            metafits_centre_freq_hz,
            metafits_filename: metafits
                .as_ref()
                .to_str()
                .expect("Metafits filename is not UTF-8 compliant")
                .to_string(),
            num_baselines,
            baselines,
            num_visibility_pols,
            visibility_pols,
        })
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
            r#"MetafitsContext (
    MWA latitude:             {mwa_lat} degrees,
    MWA longitude:            {mwa_lon} degrees
    MWA altitude:             {mwa_alt} m,

    obsid:                    {obsid},
    mode:                     {mode},

    Correlator Mode:
    fine channel resolution:  {fcw} kHz,
    integration time:         {int_time:.2} s
    num fine channels/coarse: {nfcpc},

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

    num antennas:             {n_ants},
    antennas:                 {ants:?},
    rf_inputs:                {rfs:?},

    num antenna pols:         {n_aps},
    num baselines:            {n_bls},
    baselines:                {bl01} v {bl02} to {bll1} v {bll2}
    num auto-correlations:    {n_ants},
    num cross-correlations:   {n_ccs},

    num visibility pols:      {n_vps},
    visibility pols:          {vp0}, {vp1}, {vp2}, {vp3},

    metafits FREQCENT key:    {freqcent} MHz,

    metafits filename:        {meta},
)"#,
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
            n_ants = self.num_antennas,
            ants = self.antennas,
            rfs = self.rf_inputs,
            n_aps = self.num_antenna_pols,
            n_bls = self.num_baselines,
            bl01 = self.baselines[0].antenna1_index,
            bl02 = self.baselines[0].antenna2_index,
            bll1 = self.baselines[self.num_baselines - 1].antenna1_index,
            bll2 = self.baselines[self.num_baselines - 1].antenna2_index,
            n_ccs = self.num_baselines - self.num_antennas,
            n_vps = self.num_visibility_pols,
            vp0 = self.visibility_pols[0].polarisation,
            vp1 = self.visibility_pols[1].polarisation,
            vp2 = self.visibility_pols[2].polarisation,
            vp3 = self.visibility_pols[3].polarisation,
            freqcent = self.metafits_centre_freq_hz as f64 / 1e6,
            mode = self.mode,
            fcw = self.correlator_fine_channel_width_hz as f64 / 1e3,
            nfcpc = self.num_fine_channels_per_coarse,
            int_time = self.correlator_integration_time_milliseconds as f64 / 1e3,
            meta = self.metafits_filename,
        )
    }
}

#[cfg(test)]
mod test;

/*
// num baselines:            8256,
        assert_eq!(context.num_baselines, 8256);

        // num visibility pols:      4,
        assert_eq!(context.num_visibility_pols, 4);
        // Correlator Mode:
        // fine channel resolution:  10 kHz,
        assert_eq!(context.fine_channel_width_hz, 10_000);

        // integration time:         2.00 s
        assert_eq!(context.integration_time_milliseconds, 2000);

        // num fine channels/coarse: 128,
        assert_eq!(context.num_fine_channels_per_coarse, 128);
*/
