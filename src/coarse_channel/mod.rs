// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

/*!
Structs and helper methods for coarse channel metadata
*/

use crate::gpubox_files::GpuboxTimeMap;
use crate::voltage_files::VoltageFileTimeMap;
pub mod error;
use crate::*;
pub use error::CoarseChannelError;
use std::fmt;

/// This is a struct for our coarse channels
#[derive(Clone)]
pub struct CoarseChannel {
    /// Correlator channel is 0 indexed (0..N-1)
    pub corr_chan_number: usize,

    /// Receiver channel is 0-255 in the RRI recivers
    pub rec_chan_number: usize,

    /// gpubox channel number
    /// Legacy e.g. obsid_datetime_gpuboxXX_00
    /// v2     e.g. obsid_datetime_gpuboxXXX_00
    pub gpubox_number: usize,

    /// Width of a coarse channel in Hz
    pub chan_width_hz: u32,

    /// Starting frequency of coarse channel in Hz
    pub chan_start_hz: u32,

    /// Centre frequency of coarse channel in Hz
    pub chan_centre_hz: u32,

    /// Ending frequency of coarse channel in Hz
    pub chan_end_hz: u32,
}

impl CoarseChannel {
    /// Creates a new, populated
    ///CoarseChannel struct
    ///
    /// # Arguments
    ///
    /// * `corr_chan_number` - correlator channel number. Correlator channels are numbered 0..n and represent 1st, 2nd, 3rd, etc coarse channel in the obs.
    ///
    /// * `rec_chan_number` - this is the "sky" channel number used by the receiver. For legacy the sky frequency maps to 1.28 x rec_chan_number.
    ///
    /// * `gpubox_number` - For Legacy MWA, this is 01..24. For MWAX this is 001..255. It is the number provided in the filename of the gpubox file.
    ///
    /// * `coarse_chan_width_hz` - The width in Hz of this coarse channel.
    ///
    ///
    /// # Returns
    ///
    /// * An Result containing a populated CoarseChannel struct or an Error
    ///
    pub(crate) fn new(
        corr_chan_number: usize,
        rec_chan_number: usize,
        gpubox_number: usize,
        coarse_chan_width_hz: u32,
    ) -> Self {
        let centre_chan_hz: u32 = (rec_chan_number as u32) * coarse_chan_width_hz;
        Self {
            corr_chan_number,
            rec_chan_number,
            gpubox_number,
            chan_width_hz: coarse_chan_width_hz,
            chan_centre_hz: centre_chan_hz,
            chan_start_hz: centre_chan_hz - (coarse_chan_width_hz / 2),
            chan_end_hz: centre_chan_hz + (coarse_chan_width_hz / 2),
        }
    }
    /// Takes the metafits long string of coarse channels, parses it and turns it into a vector
    /// with each element being a reciever channel number. This is the total receiver channels
    /// used in this observation.
    ///
    ///
    /// # Arguments
    ///
    /// `metafits_coarse_chans_string` - a reference to the CHANNELS long string read from the metafits file.
    ///
    /// # Returns
    ///
    /// * A vector containing all of the receiver channel numbers for this observation.
    ///
    fn get_metafits_coarse_chan_array(metafits_coarse_chans_string: &str) -> Vec<usize> {
        metafits_coarse_chans_string
            .replace(&['\'', '&'][..], "")
            .split(',')
            .map(|s| s.parse().unwrap())
            .collect()
    }

    /// Return a vector of receiver coarse channel numbers and the width of each coarse channel in Hz, given metafits and the observation bandwidth in Hz.
    ///
    /// # Arguments
    ///
    /// `metafits_fptr` - a reference to a metafits FitsFile object.
    ///
    /// `metafits_hdu` - a reference to a metafits primary HDU.
    ///    
    /// `observation_bandwidth_hz` - total bandwidth in Hz of the entire observation. If there are 24 x 1.28 MHz channels
    ///                              this would be 30.72 MHz (30,720,000 Hz)
    ///
    /// # Returns
    ///
    /// * A tuple containing: A vector of receiver channel numbers expected to be in this observation (from the metafits file),
    ///                       The width in Hz of each coarse channel
    ///
    pub(crate) fn get_metafits_coarse_channel_info(
        metafits_fptr: &mut fitsio::FitsFile,
        hdu: &fitsio::hdu::FitsHdu,
        observation_bandwidth_hz: u32,
    ) -> Result<(Vec<usize>, u32), FitsError> {
        // The coarse-channels string uses the FITS "CONTINUE" keywords. The
        // fitsio library for rust does not (appear) to handle CONTINUE keywords
        // at present, but the underlying fitsio-sys does, so we have to do FFI
        // directly.
        let coarse_chans_string =
            get_required_fits_key_long_string!(metafits_fptr, hdu, "CHANNELS")?;

        // Get the vector of coarse channels from the metafits
        let coarse_chan_vec = Self::get_metafits_coarse_chan_array(&coarse_chans_string);

        // Determine coarse channel width
        let coarse_chan_width_hz = observation_bandwidth_hz / coarse_chan_vec.len() as u32;

        Ok((coarse_chan_vec, coarse_chan_width_hz))
    }

    /// This creates a populated vector of CoarseChannel structs. It can be called 3 ways:
    /// * if `gpubox_time_map` is supplied, then the coarse channels represent actual coarse channels supplied for a CorrelatorContext.
    /// * if `voltage_time_map` is supplied, then the coarse channels represent actual coarse channels supplied for a VoltageContext.
    /// * if neither `gpubox_time_map` nor `voltage_time_map` is supplied, then the coarse channels represent the expected coarse channels supplied for a MetafitsContext.
    ///
    /// # Arguments    
    ///
    /// `corr_version` - enum representing the version of the correlator this observation was created with.
    ///
    /// `metafits_coarse_chan_vec` - A vector of receiver channel numbers expected to be in this observation (from the metafits file).
    ///
    /// `metafits_coarse_chan_width_hz` - The width in Hz of each coarse channel from the metafits.
    ///
    /// `gpubox_time_map` - An Option containing a BTreeMap detailing which timesteps exist and which gpuboxes and channels were provided by the client, or None.
    ///
    /// `voltage_time_map` - An Option containing a BTreeMap detailing which timesteps exist and which voltage files and channels were provided by the client, or None.
    ///
    /// # Returns
    ///
    /// * A tuple containing: A vector of CoarseChannel structs (limited to those are supplied by the client and are valid, unless neither `gpubox_time_map` nor
    ///                            `voltage_time_map` are provided, and the it is based on the metafits),    
    ///                       The width in Hz of each coarse channel
    ///
    pub(crate) fn populate_coarse_channels(
        corr_version: metafits_context::CorrelatorVersion,
        metafits_coarse_chan_vec: &Vec<usize>,
        metafits_coarse_chan_width_hz: u32,
        gpubox_time_map: Option<&GpuboxTimeMap>,
        voltage_time_map: Option<&VoltageFileTimeMap>,
    ) -> Result<Vec<Self>, MwalibError> {
        // Ensure we dont have a gpubox time map AND a voltage time map
        if gpubox_time_map.is_some() && voltage_time_map.is_some() {
            return Err(MwalibError::CoarseChannel(
                CoarseChannelError::BothGpuboxAndVoltageTimeMapSupplied,
            ));
        }

        let num_coarse_chans = metafits_coarse_chan_vec.len();

        // Initialise the coarse channel vector of structs
        let mut coarse_chans: Vec<CoarseChannel> = Vec::new();
        let mut first_chan_index_over_128: Option<usize> = None;
        for (i, rec_chan_number) in metafits_coarse_chan_vec.iter().enumerate() {
            // Final Correlator channel number is 0 indexed. e.g. 0..N-1
            let mut correlator_chan_number = i;

            match corr_version {
                CorrelatorVersion::Legacy | CorrelatorVersion::OldLegacy => {
                    // Legacy and Old Legacy: if receiver channel number is >128 then the order is reversed
                    if *rec_chan_number > 128 {
                        if first_chan_index_over_128.is_none() {
                            // Set this variable so we know the index where the channels reverse
                            first_chan_index_over_128 = Some(i);
                        }

                        correlator_chan_number =
                            (num_coarse_chans - 1) - (i - first_chan_index_over_128.unwrap_or(0));
                    }

                    // Before we commit to adding this coarse channel, lets ensure that the client supplied the
                    // gpubox file needed for it (if the gpu_time_map was supplied)
                    // Get the first node (which is the first timestep)
                    // Then see if a coarse channel exists based on gpubox number
                    // We add one since gpubox numbers are 1..N, while we will be recording
                    // 0..N-1
                    let gpubox_chan_number = correlator_chan_number + 1;

                    // If we have the correlator channel number, then add it to
                    // the output vector.
                    match gpubox_time_map {
                        Some(g) => {
                            if let Some((_, channel_map)) = g.iter().next() {
                                if channel_map.contains_key(&gpubox_chan_number) {
                                    coarse_chans.push(CoarseChannel::new(
                                        correlator_chan_number,
                                        *rec_chan_number,
                                        gpubox_chan_number,
                                        metafits_coarse_chan_width_hz,
                                    ))
                                }
                            }
                        }
                        _ => match voltage_time_map {
                            Some(v) => {
                                if let Some((_, channel_map)) = v.iter().next() {
                                    if channel_map.contains_key(&rec_chan_number) {
                                        coarse_chans.push(CoarseChannel::new(
                                            correlator_chan_number,
                                            *rec_chan_number,
                                            *rec_chan_number,
                                            metafits_coarse_chan_width_hz,
                                        ))
                                    }
                                }
                            }
                            _ => coarse_chans.push(CoarseChannel::new(
                                correlator_chan_number,
                                *rec_chan_number,
                                gpubox_chan_number,
                                metafits_coarse_chan_width_hz,
                            )),
                        },
                    }
                }
                CorrelatorVersion::V2 => {
                    // If we have the correlator channel number, then add it to
                    // the output vector.
                    match gpubox_time_map {
                        Some(g) => {
                            if let Some((_, channel_map)) = g.iter().next() {
                                if channel_map.contains_key(&rec_chan_number) {
                                    coarse_chans.push(CoarseChannel::new(
                                        correlator_chan_number,
                                        *rec_chan_number,
                                        *rec_chan_number,
                                        metafits_coarse_chan_width_hz,
                                    ))
                                }
                            }
                        }
                        _ => match voltage_time_map {
                            Some(v) => {
                                if let Some((_, channel_map)) = v.iter().next() {
                                    if channel_map.contains_key(&rec_chan_number) {
                                        coarse_chans.push(CoarseChannel::new(
                                            correlator_chan_number,
                                            *rec_chan_number,
                                            *rec_chan_number,
                                            metafits_coarse_chan_width_hz,
                                        ))
                                    }
                                }
                            }
                            _ => coarse_chans.push(CoarseChannel::new(
                                correlator_chan_number,
                                *rec_chan_number,
                                *rec_chan_number,
                                metafits_coarse_chan_width_hz,
                            )),
                        },
                    }
                }
            }
        }

        // Now sort the coarse channels by receiver channel number (ascending sky frequency order)
        coarse_chans.sort_by(|a, b| a.rec_chan_number.cmp(&b.rec_chan_number));

        Ok(coarse_chans)
    }
}

/// Implements fmt::Debug for
///CoarseChannel struct
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
impl fmt::Debug for CoarseChannel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "gpu={} corr={} rec={} @ {:.3} MHz",
            self.gpubox_number,
            self.corr_chan_number,
            self.rec_chan_number,
            self.chan_centre_hz as f32 / 1_000_000.
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    // Create a BTree Structure for testing
    fn get_gpubox_time_map(sub_map_keys: Vec<usize>) -> GpuboxTimeMap {
        let mut sub_map = BTreeMap::new();
        for i in sub_map_keys {
            sub_map.insert(i, (0, 1));
        }
        let mut gpubox_time_map = BTreeMap::new();
        gpubox_time_map.insert(1_381_844_923_000, sub_map);
        gpubox_time_map
    }

    // Create a BTree Structure for testing
    fn get_voltage_time_map(sub_map_keys: Vec<usize>) -> VoltageFileTimeMap {
        let mut sub_map = BTreeMap::new();
        for i in sub_map_keys {
            sub_map.insert(i, String::from("filename"));
        }
        let mut voltage_file_time_map = BTreeMap::new();
        voltage_file_time_map.insert(1_234_567_890, sub_map);
        voltage_file_time_map
    }

    #[test]
    fn test_get_metafits_coarse_chan_array() {
        assert_eq!(
            8,
            CoarseChannel::get_metafits_coarse_chan_array("0,1,2,3,127,128,129,255").len()
        );
    }

    #[test]
    /// Tests coarse channel processing for a Legacy observation where we don't have all the coarse channels
    /// What we expect from metafits is 4 coarse channels but we only get the middle 2
    /// Metafits has: 109,110,111,112
    /// User supplied gpuboxes 2 and 3 (which when 0 indexed is 1 and 2)
    /// So:
    /// 109 ==  missing
    /// 110 == gpubox02 == correlator index 1
    /// 111 == gpubox03 == correlator index 2
    /// 112 == missing
    fn test_process_coarse_chans_legacy_middle_two_gpuboxes() {
        // Create the BTree Structure for an simple test which has 2 coarse channels
        let gpubox_time_map = get_gpubox_time_map((2..=3).collect());

        // Metafits coarse channel array
        let metafits_chan_array = vec![109, 110, 111, 112];

        // Process coarse channels
        let result = CoarseChannel::populate_coarse_channels(
            CorrelatorVersion::Legacy,
            &metafits_chan_array,
            1_280_000,
            Some(&gpubox_time_map),
            None,
        );

        assert!(result.is_ok());

        let coarse_chan_array = result.unwrap();

        assert_eq!(coarse_chan_array.len(), 2);
        assert_eq!(coarse_chan_array[0].corr_chan_number, 1);
        assert_eq!(coarse_chan_array[0].rec_chan_number, 110);
        assert_eq!(coarse_chan_array[0].gpubox_number, 2);
        assert_eq!(coarse_chan_array[1].corr_chan_number, 2);
        assert_eq!(coarse_chan_array[1].rec_chan_number, 111);
        assert_eq!(coarse_chan_array[1].gpubox_number, 3);
    }

    #[test]
    /// Tests coarse channel processing when we have a legacy observation
    /// and the coarse channels span the 128 mark, thereby reversing
    /// Input from Legacy metafits:
    /// receiver channels: 126,127,128,129,130
    /// this would map to correlator indexes: 0,1,2,4,3
    fn test_process_coarse_chans_legacy_chan_reversal() {
        // Create the BTree Structure for an simple test which has 5 coarse channels
        let gpubox_time_map = get_gpubox_time_map((1..=5).collect());

        // Metafits coarse channel array
        let metafits_chan_array = vec![126, 127, 128, 129, 130];

        // Process coarse channels
        let result = CoarseChannel::populate_coarse_channels(
            CorrelatorVersion::Legacy,
            &metafits_chan_array,
            1_280_000,
            Some(&gpubox_time_map),
            None,
        );

        assert!(result.is_ok());

        let coarse_chan_array = result.unwrap();

        assert_eq!(coarse_chan_array.len(), 5);
        assert_eq!(coarse_chan_array[0].corr_chan_number, 0);
        assert_eq!(coarse_chan_array[0].rec_chan_number, 126);
        assert_eq!(coarse_chan_array[0].gpubox_number, 1);
        assert_eq!(coarse_chan_array[1].corr_chan_number, 1);
        assert_eq!(coarse_chan_array[1].rec_chan_number, 127);
        assert_eq!(coarse_chan_array[1].gpubox_number, 2);
        assert_eq!(coarse_chan_array[2].corr_chan_number, 2);
        assert_eq!(coarse_chan_array[2].rec_chan_number, 128);
        assert_eq!(coarse_chan_array[2].gpubox_number, 3);
        assert_eq!(coarse_chan_array[3].corr_chan_number, 4);
        assert_eq!(coarse_chan_array[3].rec_chan_number, 129);
        assert_eq!(coarse_chan_array[3].gpubox_number, 5);
        assert_eq!(coarse_chan_array[4].corr_chan_number, 3);
        assert_eq!(coarse_chan_array[4].rec_chan_number, 130);
        assert_eq!(coarse_chan_array[4].gpubox_number, 4);
    }

    #[test]
    /// Tests coarse channel processing for a Legacy observation where we don't have all the coarse channels
    /// What we expect from metafits is 4 coarse channels but we only get the first and last
    /// Metafits has: 109,110,111,112
    /// User supplied gpuboxes 1 and 4 (which when 0 indexed is 0 and 3)
    /// So:
    /// 109 == gpubox01 == correlator index 0
    /// 110 == missing
    /// 111 == missing
    /// 112 == gpubox04 == correlator index 3
    fn test_process_coarse_chans_legacy_first_and_last() {
        // Create the BTree Structure for an simple test which has 2 coarse channels
        let gpubox_time_map = get_gpubox_time_map(vec![1, 4]);

        // Metafits coarse channel array
        let metafits_chan_array = vec![109, 110, 111, 112];

        // Process coarse channels
        let result = CoarseChannel::populate_coarse_channels(
            CorrelatorVersion::Legacy,
            &metafits_chan_array,
            1_280_000,
            Some(&gpubox_time_map),
            None,
        );

        assert!(result.is_ok());

        let coarse_chan_array = result.unwrap();

        assert_eq!(coarse_chan_array.len(), 2);
        assert_eq!(coarse_chan_array[0].corr_chan_number, 0);
        assert_eq!(coarse_chan_array[0].rec_chan_number, 109);
        assert_eq!(coarse_chan_array[0].gpubox_number, 1);
        assert_eq!(coarse_chan_array[1].corr_chan_number, 3);
        assert_eq!(coarse_chan_array[1].rec_chan_number, 112);
        assert_eq!(coarse_chan_array[1].gpubox_number, 4);
    }

    #[test]
    /// Tests coarse channel processing when we have a MWAX observation
    /// and the coarse channels span the 128 mark. In this case we DO NOT reverse coarse channels post 128
    /// Input from MWAX metafits:
    /// receiver channels: 126,127,128,129,130
    /// this would map to correlator indexes: 0,1,2,3,4
    fn test_process_coarse_chans_mwax_no_reverse() {
        // Create the BTree Structure for an simple test which has 5 coarse channels
        let gpubox_time_map = get_gpubox_time_map(vec![126, 127, 128, 129, 130]);

        // Metafits coarse channel array
        let metafits_chan_array = vec![126, 127, 128, 129, 130];

        // Process coarse channels
        let result = CoarseChannel::populate_coarse_channels(
            CorrelatorVersion::V2,
            &metafits_chan_array,
            1_280_000,
            Some(&gpubox_time_map),
            None,
        );

        assert!(result.is_ok());

        let coarse_chan_array = result.unwrap();

        assert_eq!(coarse_chan_array.len(), 5);
        assert_eq!(coarse_chan_array[0].corr_chan_number, 0);
        assert_eq!(coarse_chan_array[0].rec_chan_number, 126);
        assert_eq!(coarse_chan_array[0].gpubox_number, 126);
        assert_eq!(coarse_chan_array[1].corr_chan_number, 1);
        assert_eq!(coarse_chan_array[1].rec_chan_number, 127);
        assert_eq!(coarse_chan_array[1].gpubox_number, 127);
        assert_eq!(coarse_chan_array[2].corr_chan_number, 2);
        assert_eq!(coarse_chan_array[2].rec_chan_number, 128);
        assert_eq!(coarse_chan_array[2].gpubox_number, 128);
        assert_eq!(coarse_chan_array[3].corr_chan_number, 3);
        assert_eq!(coarse_chan_array[3].rec_chan_number, 129);
        assert_eq!(coarse_chan_array[3].gpubox_number, 129);
        assert_eq!(coarse_chan_array[4].corr_chan_number, 4);
        assert_eq!(coarse_chan_array[4].rec_chan_number, 130);
        assert_eq!(coarse_chan_array[4].gpubox_number, 130);
    }

    #[test]
    /// This test exposed a bug which is triggered when a legacy observation has
    /// all coarse channel numbers > 128 (typical for EoR).
    fn test_process_coarse_chans_legacy_eor() {
        let gpubox_time_map = get_gpubox_time_map((1..=3).collect());
        let metafits_chan_array: Vec<_> = (133..=135).collect();
        let channel_width = 1_280_000;

        // Process coarse channels
        let result = CoarseChannel::populate_coarse_channels(
            CorrelatorVersion::Legacy,
            &metafits_chan_array,
            channel_width,
            Some(&gpubox_time_map),
            None,
        );

        assert!(result.is_ok());

        let coarse_chan_array = result.unwrap();

        assert_eq!(coarse_chan_array.len(), 3);
        assert_eq!(coarse_chan_array[0].corr_chan_number, 2);
        assert_eq!(coarse_chan_array[0].rec_chan_number, 133);
        assert_eq!(coarse_chan_array[0].gpubox_number, 3);
        assert_eq!(coarse_chan_array[1].corr_chan_number, 1);
        assert_eq!(coarse_chan_array[1].rec_chan_number, 134);
        assert_eq!(coarse_chan_array[1].gpubox_number, 2);
        assert_eq!(coarse_chan_array[2].corr_chan_number, 0);
        assert_eq!(coarse_chan_array[2].rec_chan_number, 135);
        assert_eq!(coarse_chan_array[2].gpubox_number, 1);
    }

    #[test]
    fn test_process_coarse_chans_no_time_maps_legacy() {
        let metafits_chan_array: Vec<_> = (133..=135).collect();
        let channel_width = 1_280_000;

        // Process coarse channels
        let result = CoarseChannel::populate_coarse_channels(
            CorrelatorVersion::Legacy,
            &metafits_chan_array,
            channel_width,
            None,
            None,
        );

        assert!(result.is_ok());

        let coarse_chan_array = result.unwrap();

        assert_eq!(coarse_chan_array.len(), 3);
    }

    #[test]
    fn test_process_coarse_chans_no_time_maps_mwax_v2() {
        let metafits_chan_array: Vec<_> = (133..=135).collect();
        let channel_width = 1_280_000;

        // Process coarse channels
        let result = CoarseChannel::populate_coarse_channels(
            CorrelatorVersion::V2,
            &metafits_chan_array,
            channel_width,
            None,
            None,
        );

        assert!(result.is_ok());

        let coarse_chan_array = result.unwrap();

        assert_eq!(coarse_chan_array.len(), 3);
    }

    #[test]
    fn test_process_coarse_chans_both_time_maps() {
        let gpubox_time_map = get_gpubox_time_map((1..=3).collect());
        let voltage_time_map = get_voltage_time_map((1..=3).collect());
        let metafits_chan_array: Vec<_> = (133..=135).collect();
        let channel_width = 1_280_000;

        // Process coarse channels for legacy
        let result1 = CoarseChannel::populate_coarse_channels(
            CorrelatorVersion::Legacy,
            &metafits_chan_array,
            channel_width,
            Some(&gpubox_time_map),
            Some(&voltage_time_map),
        );

        assert!(matches!(
            result1.unwrap_err(),
            MwalibError::CoarseChannel(CoarseChannelError::BothGpuboxAndVoltageTimeMapSupplied)
        ));

        // v2
        let result2 = CoarseChannel::populate_coarse_channels(
            CorrelatorVersion::V2,
            &metafits_chan_array,
            channel_width,
            Some(&gpubox_time_map),
            Some(&voltage_time_map),
        );

        assert!(matches!(
            result2.unwrap_err(),
            MwalibError::CoarseChannel(CoarseChannelError::BothGpuboxAndVoltageTimeMapSupplied)
        ));
    }

    #[test]
    fn test_coarse_chan_debug() {
        let cc = CoarseChannel::new(1, 109, 2, 1_280_000);

        assert_eq!(format!("{:?}", cc), "gpu=2 corr=1 rec=109 @ 139.520 MHz");
    }
}
