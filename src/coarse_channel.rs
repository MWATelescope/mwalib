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
pub struct mwalibCoarseChannel {
    // Correlator channel is 0 indexed
    pub correlator_channel_number: usize,

    // Receiver channel is 0-255 in the RRI recivers
    pub receiver_channel_number: usize,

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
        channel_width_hz: u32,
    ) -> mwalibCoarseChannel {
        let centre_chan_hz: u32 = (receiver_channel_number as u32) * channel_width_hz;

        mwalibCoarseChannel {
            correlator_channel_number,
            receiver_channel_number,
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
        corr_version: &context::CorrelatorVersion,
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
        corr_version: &context::CorrelatorVersion,
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
        for (i, rec_channel_number) in coarse_channel_vec.into_iter().enumerate() {
            let mut correlator_channel_number = i;

            if *corr_version == CorrelatorVersion::Legacy
                || *corr_version == CorrelatorVersion::OldLegacy
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
            }

            // Before we commit to adding this coarse channel, lets ensure that the client supplied the
            // gpubox file needed for it
            // Get the first node (which is the first timestep)
            // Then see if a coarse channel exists based on gpubox number
            if gpubox_time_map
                .iter()
                .next() // this gets us the first item
                .unwrap()
                .1 // get the second item from tuple (u64, BTreeMap)
                .contains_key(&correlator_channel_number)
            // see if we have the correlator channel number
            {
                coarse_channels.push(mwalibCoarseChannel::new(
                    correlator_channel_number,
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
            "corr={} rec={} @ {:.3} MHz",
            self.correlator_channel_number,
            self.receiver_channel_number,
            self.channel_centre_hz as f32 / 1_000_000.
        )
    }
}
