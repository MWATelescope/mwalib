// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Structs and helper methods for coarse channel metadata

use crate::gpubox_files::GpuboxTimeMap;
use crate::voltage_files::VoltageFileTimeMap;
pub mod error;
use crate::*;
use error::CoarseChannelError;
use fitsio::{hdu::FitsHdu, FitsFile};
use std::fmt;

#[cfg(any(feature = "python", feature = "python-stubgen"))]
use pyo3::prelude::*;
#[cfg(feature = "python-stubgen")]
use pyo3_stub_gen_derive::gen_stub_pyclass;

pub mod ffi;

#[cfg(test)]
mod test;

/// This is a struct for coarse channels
#[cfg_attr(feature = "python-stubgen", gen_stub_pyclass)]
#[cfg_attr(
    any(feature = "python", feature = "python-stubgen"),
    pyclass(get_all, set_all)
)]
#[derive(Clone, PartialEq)]
pub struct CoarseChannel {
    /// Correlator channel is 0 indexed (0..N-1)
    pub corr_chan_number: usize,

    /// Receiver channel is 0-255 in the RRI recivers
    pub rec_chan_number: usize,

    /// gpubox channel number
    /// This is better described as the identifier which would be in the filename of visibility files
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
        let coarse_chan_centre_hz: u32 = (rec_chan_number as u32) * coarse_chan_width_hz;

        Self {
            corr_chan_number,
            rec_chan_number,
            gpubox_number,
            chan_width_hz: coarse_chan_width_hz,
            chan_centre_hz: coarse_chan_centre_hz,
            chan_start_hz: coarse_chan_centre_hz - (coarse_chan_width_hz / 2),
            chan_end_hz: coarse_chan_centre_hz + (coarse_chan_width_hz / 2),
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
    ///   The width in Hz of each coarse channel
    ///
    pub(crate) fn get_metafits_coarse_channel_info(
        metafits_fptr: &mut FitsFile,
        hdu: &FitsHdu,
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
    /// `mwa_version` - enum representing the version of the correlator this observation was created with.
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
    ///   `voltage_time_map` are provided, and the it is based on the metafits),
    ///   The width in Hz of each coarse channel
    ///
    pub(crate) fn populate_coarse_channels(
        mwa_version: types::MWAVersion,
        metafits_coarse_chan_vec: &[usize],
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

        // get the count of metafits coarse channels
        let num_metafits_coarse_chans = metafits_coarse_chan_vec.len();

        // Initialise the coarse channel vector of structs
        let mut coarse_chans: Vec<CoarseChannel> = Vec::new();
        let mut first_chan_index_over_128: Option<usize> = None;
        for (i, rec_chan_number) in metafits_coarse_chan_vec.iter().enumerate() {
            // Final Correlator channel number is 0 indexed. e.g. 0..N-1
            let mut correlator_chan_number = i;

            match mwa_version {
                MWAVersion::CorrLegacy
                | MWAVersion::CorrOldLegacy
                | MWAVersion::VCSLegacyRecombined => {
                    // Legacy and Old Legacy: if receiver channel number is >128 then the order is reversed
                    if *rec_chan_number > 128 {
                        if first_chan_index_over_128.is_none() {
                            // Set this variable so we know the index where the channels reverse
                            first_chan_index_over_128 = Some(i);
                        }

                        correlator_chan_number = (num_metafits_coarse_chans - 1)
                            - (i - first_chan_index_over_128.unwrap_or(0));
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
                                    if channel_map.contains_key(rec_chan_number) {
                                        coarse_chans.push(CoarseChannel::new(
                                            correlator_chan_number,
                                            *rec_chan_number,
                                            *rec_chan_number,
                                            metafits_coarse_chan_width_hz,
                                        ))
                                    }
                                }
                            }
                            // If no timemap has been passed in, we are populating based on metafits only.
                            // Need to do different behaviour for coarse channels for LegacyVCS vs CorrLegacy and CorrOldLegacy
                            _ => match mwa_version {
                                MWAVersion::CorrLegacy | MWAVersion::CorrOldLegacy => coarse_chans
                                    .push(CoarseChannel::new(
                                        correlator_chan_number,
                                        *rec_chan_number,
                                        gpubox_chan_number,
                                        metafits_coarse_chan_width_hz,
                                    )),
                                MWAVersion::VCSLegacyRecombined => {
                                    coarse_chans.push(CoarseChannel::new(
                                        correlator_chan_number,
                                        *rec_chan_number,
                                        *rec_chan_number,
                                        metafits_coarse_chan_width_hz,
                                    ))
                                }
                                _ => {} // CorrMWAXv2 and VCSMWAXv2 wiil never get here- they are in the below section
                            },
                        },
                    }
                }
                MWAVersion::CorrMWAXv2 | MWAVersion::VCSMWAXv2 => {
                    // If we have the correlator channel number, then add it to
                    // the output vector.
                    match gpubox_time_map {
                        Some(g) => {
                            if let Some((_, channel_map)) = g.iter().next() {
                                if channel_map.contains_key(rec_chan_number) {
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
                                    if channel_map.contains_key(rec_chan_number) {
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
        coarse_chans.sort_by_key(|c| c.rec_chan_number);

        Ok(coarse_chans)
    }

    /// This creates a populated vector of indices from the passed in `all_coarse_chans' slice of CoarseChannels based on the
    /// coarse_chan_identifiers vector of coarse channel identifers we pass in.
    ///
    /// # Arguments
    ///
    /// * `all_coarse_chans` - Reference to a slice containing all the coarse channels
    ///
    /// * `coarse_chan_identifiers` - Vector or coarse channel identifiers we want to find the indices for.
    ///
    /// # Returns
    ///
    /// * A populated vector of coarse channel indices based on the passed in identifiers.
    ///
    pub(crate) fn get_coarse_chan_indicies(
        all_coarse_chans: &[Self],
        coarse_chan_identifiers: &[usize],
    ) -> Vec<usize> {
        let mut coarse_chan_indices: Vec<usize> = coarse_chan_identifiers
            .iter()
            .map(|chan_identifier| {
                all_coarse_chans
                    .iter()
                    .position(|coarse_chan| coarse_chan.gpubox_number == *chan_identifier)
                    .unwrap()
            })
            .collect::<Vec<usize>>();

        coarse_chan_indices.sort_unstable();

        coarse_chan_indices
    }

    /// Calculate the centre frequency of each fine channel of the provided coarse channels.
    ///
    ///
    /// # Arguments
    ///
    /// * `mwa_version` - The version of the MWA is in use.
    ///
    /// * `coarse_channels` - Vector of populated Coarse Channels.
    ///
    /// * `fine_chan_width_hz` - Fine channel width in Hz.
    ///
    /// * `num_fine_chans_per_coarse` - Number of fine channels per coarse channel.
    ///
    /// # Returns
    ///
    /// * The centre frequency of the first fine channel of the coarse channel.
    ///
    pub fn get_fine_chan_centres_array_hz(
        mwa_version: MWAVersion,
        coarse_channels: &[CoarseChannel],
        fine_chan_width_hz: u32,
        num_fine_chans_per_coarse: usize,
    ) -> Vec<f64> {
        Self::get_fine_chan_centres_array_hz_inner(
            mwa_version,
            coarse_channels.iter(),
            fine_chan_width_hz,
            num_fine_chans_per_coarse,
        )
    }

    /// Calculate the centre frequency of each fine channel of the provided
    /// coarse channels. This function actually does the work of the public
    /// function above and mostly exists to provide a generic interface (an
    /// iterator rather than a slice, allowing allocations to be avoided).
    pub(crate) fn get_fine_chan_centres_array_hz_inner<'a>(
        mwa_version: MWAVersion,
        coarse_channels: impl Iterator<Item = &'a CoarseChannel>,
        fine_chan_width_hz: u32,
        num_fine_chans_per_coarse: usize,
    ) -> Vec<f64> {
        // Firstly calculate the offset
        // For Legacy MWA, the offset is only needed if the fine channel width is 20 or 40kHz.
        let offset_hz = match mwa_version {
            MWAVersion::CorrLegacy
            | MWAVersion::CorrOldLegacy
            | MWAVersion::VCSLegacyRecombined => match num_fine_chans_per_coarse {
                64 => 5_000.0,  // 20 kHz corr mode needs a 5 kHz offset applied
                32 => 15_000.0, // 40 kHz corr mode needs a 15 kHz offset applied
                _ => 0.0,       // other modes (10kHz) does not need any offset applied
            },
            MWAVersion::CorrMWAXv2 | MWAVersion::VCSMWAXv2 => 0.0,
        };

        // We need a factor based on whether the number of fine channels per coarse is even or odd
        let odd_even_adjustment: f64 = match num_fine_chans_per_coarse % 2 == 0 {
            true => 0.0,  // Even
            false => 0.5, // Odd
        };

        // Return a vector of f64s which are the fine channel centre frequencies for all the fine channels in [coarse_channels]
        let return_vec: Vec<f64> = coarse_channels
            .into_iter()
            .flat_map(|coarse_chan| {
                (0..num_fine_chans_per_coarse).map(move |fine_chan_idx| {
                    coarse_chan.chan_start_hz as f64
                        + ((fine_chan_idx as f64 + odd_even_adjustment) * fine_chan_width_hz as f64)
                        + offset_hz
                })
            })
            .collect();

        return_vec
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
