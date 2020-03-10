// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

/*!
Structs and helper methods for coarse channel metadata
*/
use crate::fits_read::*;
use crate::*;
use std::collections::BTreeMap;
use std::fmt;

/// This is a struct for our coarse channels
#[allow(non_camel_case_types)]
#[derive(Clone)]
pub struct mwalibCoarseChannel {
    // Correlator channel is 0 indexed (0..N-1)
    pub correlator_channel_number: usize,

    // Receiver channel is 0-255 in the RRI recivers
    pub receiver_channel_number: usize,

    // gpubox channel number
    // Legacy e.g. obsid_datetime_gpuboxXX_00
    // v2     e.g. obsid_datetime_gpuboxXXX_00
    pub gpubox_number: usize,

    // Width of a coarse channel in Hz
    pub channel_width_hz: u32,

    // Starting frequency of coarse channel in Hz
    pub channel_start_hz: u32,

    // Centre frequency of coarse channel in Hz
    pub channel_centre_hz: u32,

    // Ending frequency of coarse channel in Hz
    pub channel_end_hz: u32,
}

impl mwalibCoarseChannel {
    pub fn new(
        correlator_channel_number: usize,
        receiver_channel_number: usize,
        gpubox_number: usize,
        channel_width_hz: u32,
    ) -> mwalibCoarseChannel {
        let centre_chan_hz: u32 = (receiver_channel_number as u32) * channel_width_hz;

        mwalibCoarseChannel {
            correlator_channel_number,
            receiver_channel_number,
            gpubox_number,
            channel_width_hz,
            channel_centre_hz: centre_chan_hz,
            channel_start_hz: centre_chan_hz - (channel_width_hz / 2),
            channel_end_hz: centre_chan_hz + (channel_width_hz / 2),
        }
    }

    // Takes a fits pointer to the metafits file and retrieves the long string for CHANNELS
    pub fn get_metafits_coarse_channel_string(
        metafits_fptr: &mut fitsio::FitsFile,
    ) -> Result<String, ErrorKind> {
        // Read the long string.
        let (status, coarse_channels_string) =
            unsafe { get_fits_long_string(metafits_fptr.as_raw(), "CHANNELS") };
        if status != 0 {
            return Err(ErrorKind::Custom(
                "mwalibContext::new: get_fits_long_string failed".to_string(),
            ));
        }

        Ok(coarse_channels_string)
    }

    // Takes the metafits long string of coarse channels and turns it into a vector
    // with each element being a reciever channel number
    pub fn get_metafits_coarse_channel_array(metafits_coarse_channels_string: &str) -> Vec<usize> {
        metafits_coarse_channels_string
            .replace(&['\'', '&'][..], "")
            .split(',')
            .map(|s| s.parse().unwrap())
            .collect()
    }

    pub fn populate_coarse_channels(
        metafits_fptr: &mut fitsio::FitsFile,
        corr_version: context::CorrelatorVersion,
        observation_bandwidth_hz: u32,
        gpubox_time_map: &BTreeMap<u64, BTreeMap<usize, (usize, usize)>>,
    ) -> Result<(Vec<mwalibCoarseChannel>, usize, u32), ErrorKind> {
        // The coarse-channels string uses the FITS "CONTINUE" keywords. The
        // fitsio library for rust does not (appear) to handle CONTINUE keywords
        // at present, but the underlying fitsio-sys does, so we have to do FFI
        // directly.
        let coarse_channels_string =
            mwalibCoarseChannel::get_metafits_coarse_channel_string(metafits_fptr)?;

        // Get the vector of coarse channels from the metafits
        let coarse_channel_vec =
            mwalibCoarseChannel::get_metafits_coarse_channel_array(&coarse_channels_string);

        // Process the coarse channels, matching them to frequencies and which gpuboxes are provided
        let (coarse_channels, num_coarse_channels, coarse_channel_width_hz) =
            mwalibCoarseChannel::process_coarse_channels(
                corr_version,
                observation_bandwidth_hz,
                &coarse_channel_vec,
                gpubox_time_map,
            );

        Ok((
            coarse_channels,
            num_coarse_channels,
            coarse_channel_width_hz,
        ))
    }

    fn process_coarse_channels(
        corr_version: context::CorrelatorVersion,
        observation_bandwidth_hz: u32,
        coarse_channel_vec: &Vec<usize>,
        gpubox_time_map: &BTreeMap<u64, BTreeMap<usize, (usize, usize)>>,
    ) -> (Vec<mwalibCoarseChannel>, usize, u32) {
        // How many coarse channels should there be (from the metafits)
        // NOTE: this will NOT always be equal to the number of gpubox files we get
        let mut num_coarse_channels = coarse_channel_vec.len();

        // Determine coarse channel width
        let coarse_channel_width_hz = observation_bandwidth_hz / num_coarse_channels as u32;

        // Initialise the coarse channel vector of structs
        let mut coarse_channels: Vec<mwalibCoarseChannel> = Vec::new();
        let mut first_chan_index_over_128: usize = 0;
        for (i, rec_channel_number) in coarse_channel_vec.iter().enumerate() {
            // Final Correlator channel number is 0 indexed. e.g. 0..N-1
            let mut correlator_channel_number = i;

            if corr_version == CorrelatorVersion::Legacy
                || corr_version == CorrelatorVersion::OldLegacy
            {
                // Legacy and Old Legacy: if receiver channel number is >128 then the order is reversed
                if *rec_channel_number > 128 {
                    if first_chan_index_over_128 == 0 {
                        // Set this variable so we know the index where the channels reverse
                        first_chan_index_over_128 = i;
                    }

                    correlator_channel_number =
                        (num_coarse_channels - 1) - (i - first_chan_index_over_128);
                }

                // Before we commit to adding this coarse channel, lets ensure that the client supplied the
                // gpubox file needed for it
                // Get the first node (which is the first timestep)
                // Then see if a coarse channel exists based on gpubox number
                // We add one since gpubox numbers are 1..N, while we will be recording
                // 0..N-1
                let gpubox_channel_number = correlator_channel_number + 1;

                if gpubox_time_map
                    .iter()
                    .next() // this gets us the first item
                    .unwrap()
                    .1 // get the second item from tuple (u64, BTreeMap)
                    .contains_key(&gpubox_channel_number)
                // see if we have the correlator channel number
                {
                    coarse_channels.push(mwalibCoarseChannel::new(
                        correlator_channel_number,
                        *rec_channel_number,
                        gpubox_channel_number,
                        coarse_channel_width_hz,
                    ));
                }
            } else if gpubox_time_map
                .iter()
                .next() // this gets us the first item
                .unwrap()
                .1 // get the second item from tuple (u64, BTreeMap)
                .contains_key(&rec_channel_number)
            // see if we have the correlator channel number
            {
                coarse_channels.push(mwalibCoarseChannel::new(
                    correlator_channel_number,
                    *rec_channel_number,
                    *rec_channel_number,
                    coarse_channel_width_hz,
                ));
            }
        }

        // Now sort the coarse channels by receiver channel number (ascending sky frequency order)
        coarse_channels.sort_by(|a, b| a.receiver_channel_number.cmp(&b.receiver_channel_number));

        // Update num coarse channels as we may have a different number now that we have checked the gpubox files
        num_coarse_channels = coarse_channels.len();

        (
            coarse_channels,
            num_coarse_channels,
            coarse_channel_width_hz,
        )
    }
}

impl fmt::Debug for mwalibCoarseChannel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "gpu={} corr={} rec={} @ {:.3} MHz",
            self.gpubox_number,
            self.correlator_channel_number,
            self.receiver_channel_number,
            self.channel_centre_hz as f32 / 1_000_000.
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_metafits_coarse_channel_array() {
        assert_eq!(
            8,
            mwalibCoarseChannel::get_metafits_coarse_channel_array("0,1,2,3,127,128,129,255").len()
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
    fn test_process_coarse_channels_legacy_middle_two_gpuboxes() {
        // Create the BTree Structure for an simple test which has 2 coarse channels
        let mut gpubox_time_map: BTreeMap<u64, std::collections::BTreeMap<usize, (usize, usize)>> =
            BTreeMap::new();
        gpubox_time_map
            .entry(1_381_844_923_000)
            .or_insert_with(BTreeMap::new)
            .entry(2)
            .or_insert((0, 1));

        gpubox_time_map
            .entry(1_381_844_923_000)
            .or_insert_with(BTreeMap::new)
            .entry(3)
            .or_insert((0, 1));

        // Metafits coarse channel array
        let mut metafits_channel_array: Vec<usize> = Vec::new();
        metafits_channel_array.push(109);
        metafits_channel_array.push(110);
        metafits_channel_array.push(111);
        metafits_channel_array.push(112);

        // Process coarse channels
        let (coarse_channel_array, coarse_channel_count, coarse_channel_width_hz) =
            mwalibCoarseChannel::process_coarse_channels(
                CorrelatorVersion::Legacy,
                1_280_000 * 4,
                &metafits_channel_array,
                &gpubox_time_map,
            );
        assert_eq!(coarse_channel_array.len(), 2);
        assert_eq!(coarse_channel_count, 2);
        assert_eq!(coarse_channel_width_hz, 1_280_000);
        assert_eq!(coarse_channel_array[0].correlator_channel_number, 1);
        assert_eq!(coarse_channel_array[0].receiver_channel_number, 110);
        assert_eq!(coarse_channel_array[0].gpubox_number, 2);
        assert_eq!(coarse_channel_array[1].correlator_channel_number, 2);
        assert_eq!(coarse_channel_array[1].receiver_channel_number, 111);
        assert_eq!(coarse_channel_array[1].gpubox_number, 3);
    }

    #[test]
    /// Tests coarse channel processing when we have a legacy observation
    /// and the coarse channels span the 128 mark, thereby reversing
    /// Input from Legacy metafits:
    /// receiver channels: 126,127,128,129,130
    /// this would map to correlator indexes: 0,1,2,4,3
    fn test_process_coarse_channels_legacy_channel_reversal() {
        // Create the BTree Structure for an simple test which has 5 coarse channels
        let mut gpubox_time_map: BTreeMap<u64, std::collections::BTreeMap<usize, (usize, usize)>> =
            BTreeMap::new();
        gpubox_time_map
            .entry(1_381_844_923_000)
            .or_insert_with(BTreeMap::new)
            .entry(1)
            .or_insert((0, 1));

        gpubox_time_map
            .entry(1_381_844_923_000)
            .or_insert_with(BTreeMap::new)
            .entry(2)
            .or_insert((0, 1));

        gpubox_time_map
            .entry(1_381_844_923_000)
            .or_insert_with(BTreeMap::new)
            .entry(3)
            .or_insert((0, 1));

        gpubox_time_map
            .entry(1_381_844_923_000)
            .or_insert_with(BTreeMap::new)
            .entry(4)
            .or_insert((0, 1));

        gpubox_time_map
            .entry(1_381_844_923_000)
            .or_insert_with(BTreeMap::new)
            .entry(5)
            .or_insert((0, 1));

        // Metafits coarse channel array
        let mut metafits_channel_array: Vec<usize> = Vec::new();
        metafits_channel_array.push(126);
        metafits_channel_array.push(127);
        metafits_channel_array.push(128);
        metafits_channel_array.push(129);
        metafits_channel_array.push(130);

        // Process coarse channels
        let (coarse_channel_array, coarse_channel_count, coarse_channel_width_hz) =
            mwalibCoarseChannel::process_coarse_channels(
                CorrelatorVersion::Legacy,
                1_280_000 * 5,
                &metafits_channel_array,
                &gpubox_time_map,
            );
        assert_eq!(coarse_channel_array.len(), 5);
        assert_eq!(coarse_channel_count, 5);
        assert_eq!(coarse_channel_width_hz, 1_280_000);
        assert_eq!(coarse_channel_array[0].correlator_channel_number, 0);
        assert_eq!(coarse_channel_array[0].receiver_channel_number, 126);
        assert_eq!(coarse_channel_array[0].gpubox_number, 1);
        assert_eq!(coarse_channel_array[1].correlator_channel_number, 1);
        assert_eq!(coarse_channel_array[1].receiver_channel_number, 127);
        assert_eq!(coarse_channel_array[1].gpubox_number, 2);
        assert_eq!(coarse_channel_array[2].correlator_channel_number, 2);
        assert_eq!(coarse_channel_array[2].receiver_channel_number, 128);
        assert_eq!(coarse_channel_array[2].gpubox_number, 3);
        assert_eq!(coarse_channel_array[3].correlator_channel_number, 4);
        assert_eq!(coarse_channel_array[3].receiver_channel_number, 129);
        assert_eq!(coarse_channel_array[3].gpubox_number, 5);
        assert_eq!(coarse_channel_array[4].correlator_channel_number, 3);
        assert_eq!(coarse_channel_array[4].receiver_channel_number, 130);
        assert_eq!(coarse_channel_array[4].gpubox_number, 4);
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
    fn test_process_coarse_channels_legacy_first_and_last() {
        // Create the BTree Structure for an simple test which has 2 coarse channels
        let mut gpubox_time_map: BTreeMap<u64, std::collections::BTreeMap<usize, (usize, usize)>> =
            BTreeMap::new();
        gpubox_time_map
            .entry(1_381_844_923_000)
            .or_insert_with(BTreeMap::new)
            .entry(1)
            .or_insert((0, 1));

        gpubox_time_map
            .entry(1_381_844_923_000)
            .or_insert_with(BTreeMap::new)
            .entry(4)
            .or_insert((0, 1));

        // Metafits coarse channel array
        let mut metafits_channel_array: Vec<usize> = Vec::new();
        metafits_channel_array.push(109);
        metafits_channel_array.push(110);
        metafits_channel_array.push(111);
        metafits_channel_array.push(112);

        // Process coarse channels
        let (coarse_channel_array, coarse_channel_count, coarse_channel_width_hz) =
            mwalibCoarseChannel::process_coarse_channels(
                CorrelatorVersion::Legacy,
                1_280_000 * 4,
                &metafits_channel_array,
                &gpubox_time_map,
            );
        assert_eq!(coarse_channel_array.len(), 2);
        assert_eq!(coarse_channel_count, 2);
        assert_eq!(coarse_channel_width_hz, 1_280_000);
        assert_eq!(coarse_channel_array[0].correlator_channel_number, 0);
        assert_eq!(coarse_channel_array[0].receiver_channel_number, 109);
        assert_eq!(coarse_channel_array[0].gpubox_number, 1);
        assert_eq!(coarse_channel_array[1].correlator_channel_number, 3);
        assert_eq!(coarse_channel_array[1].receiver_channel_number, 112);
        assert_eq!(coarse_channel_array[1].gpubox_number, 4);
    }

    #[test]
    /// Tests coarse channel processing when we have a MWAX observation
    /// and the coarse channels span the 128 mark. In this case we DO NOT reverse coarse channels post 128
    /// Input from MWAX metafits:
    /// receiver channels: 126,127,128,129,130
    /// this would map to correlator indexes: 0,1,2,3,4
    fn test_process_coarse_channels_mwax_no_reverse() {
        // Create the BTree Structure for an simple test which has 5 coarse channels
        let mut gpubox_time_map: BTreeMap<u64, std::collections::BTreeMap<usize, (usize, usize)>> =
            BTreeMap::new();
        gpubox_time_map
            .entry(1_381_844_923_000)
            .or_insert_with(BTreeMap::new)
            .entry(126)
            .or_insert((0, 1));

        gpubox_time_map
            .entry(1_381_844_923_000)
            .or_insert_with(BTreeMap::new)
            .entry(127)
            .or_insert((0, 1));

        gpubox_time_map
            .entry(1_381_844_923_000)
            .or_insert_with(BTreeMap::new)
            .entry(128)
            .or_insert((0, 1));

        gpubox_time_map
            .entry(1_381_844_923_000)
            .or_insert_with(BTreeMap::new)
            .entry(129)
            .or_insert((0, 1));

        gpubox_time_map
            .entry(1_381_844_923_000)
            .or_insert_with(BTreeMap::new)
            .entry(130)
            .or_insert((0, 1));

        // Metafits coarse channel array
        let mut metafits_channel_array: Vec<usize> = Vec::new();
        metafits_channel_array.push(126);
        metafits_channel_array.push(127);
        metafits_channel_array.push(128);
        metafits_channel_array.push(129);
        metafits_channel_array.push(130);

        // Process coarse channels
        let (coarse_channel_array, coarse_channel_count, coarse_channel_width_hz) =
            mwalibCoarseChannel::process_coarse_channels(
                CorrelatorVersion::V2,
                1_280_000 * 5,
                &metafits_channel_array,
                &gpubox_time_map,
            );
        assert_eq!(coarse_channel_array.len(), 5);
        assert_eq!(coarse_channel_count, 5);
        assert_eq!(coarse_channel_width_hz, 1_280_000);
        assert_eq!(coarse_channel_array[0].correlator_channel_number, 0);
        assert_eq!(coarse_channel_array[0].receiver_channel_number, 126);
        assert_eq!(coarse_channel_array[0].gpubox_number, 126);
        assert_eq!(coarse_channel_array[1].correlator_channel_number, 1);
        assert_eq!(coarse_channel_array[1].receiver_channel_number, 127);
        assert_eq!(coarse_channel_array[1].gpubox_number, 127);
        assert_eq!(coarse_channel_array[2].correlator_channel_number, 2);
        assert_eq!(coarse_channel_array[2].receiver_channel_number, 128);
        assert_eq!(coarse_channel_array[2].gpubox_number, 128);
        assert_eq!(coarse_channel_array[3].correlator_channel_number, 3);
        assert_eq!(coarse_channel_array[3].receiver_channel_number, 129);
        assert_eq!(coarse_channel_array[3].gpubox_number, 129);
        assert_eq!(coarse_channel_array[4].correlator_channel_number, 4);
        assert_eq!(coarse_channel_array[4].receiver_channel_number, 130);
        assert_eq!(coarse_channel_array[4].gpubox_number, 130);
    }
}
