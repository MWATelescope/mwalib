// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

/// Given an observation's data, verify that `mwalib` is functioning correctly
/// by printing the sum of the visibilities belonging to the first fine channel
/// from each baseline.
///
/// Works only on MWAX data for now.
use anyhow::*;
use structopt::StructOpt;

use mwalib::*;

#[cfg(not(tarpaulin_include))]
#[derive(StructOpt, Debug)]
#[structopt(name = "mwalib-sum-first-fine-channel-gpubox-hdus", author)]
struct Opt {
    /// Path to the metafits file.
    #[structopt(short, long, parse(from_os_str))]
    metafits: std::path::PathBuf,

    /// Paths to the gpubox files.
    #[structopt(name = "GPUBOX FILE", parse(from_os_str))]
    files: Vec<std::path::PathBuf>,
}

#[cfg(not(tarpaulin_include))]
#[allow(clippy::needless_range_loop)] // Ignoring this, as it is a false positive
fn main() -> Result<(), anyhow::Error> {
    let opts = Opt::from_args();

    let context = CorrelatorContext::new(&opts.metafits, &opts.files)?;
    if context.mwa_version != MWAVersion::CorrMWAXv2 {
        bail!("Input data is not MWAX data; exiting.");
    }
    let floats_per_fine_chan = context.metafits_context.num_visibility_pols * 2;

    let mut sum: f64 = 0.0;
    for timestep_index in 0..context.num_timesteps {
        for coarse_chan_index in 0..context.num_coarse_chans {
            let data = context.read_by_baseline(timestep_index, coarse_chan_index)?;
            for baseline in 0..context.metafits_context.num_baselines {
                // We want the first fine chan for each baseline
                let start_index = baseline
                    * (context.metafits_context.num_corr_fine_chans_per_coarse
                        * floats_per_fine_chan);
                let end_index = start_index + floats_per_fine_chan;

                assert_eq!(end_index - start_index, 8);

                if timestep_index == 0 {
                    println!("{} {}-{}", baseline, start_index, end_index);
                }

                for index in start_index..end_index {
                    sum += data[index] as f64;
                }
            }
        }
    }

    println!("Sum of every first fine channel: {} == {:+e}", sum, sum);

    Ok(())
}
