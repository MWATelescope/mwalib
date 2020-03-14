/// Given an observation's data, verify that `mwalib` is functioning correctly
/// by printing the sum of the visibilities belonging to the first fine channel
/// from each baseline.
///
/// Works only on MWAX data for now.
use anyhow::*;
use structopt::StructOpt;

use mwalib::*;

#[derive(StructOpt, Debug)]
#[structopt(name = "mwalib-sum-first-fine-channel-gpubox-hdus", author)]
struct Opt {
    /// Path to the metafits file.
    #[structopt(short, long)]
    metafits: String,

    /// Paths to the gpubox files.
    #[structopt(name = "GPUBOX FILE")]
    files: Vec<String>,
}

#[allow(clippy::needless_range_loop)]  // Ignoring this, as it is a false positive
fn main() -> Result<(), anyhow::Error> {
    let opts = Opt::from_args();

    let mut context = mwalibContext::new(&opts.metafits, &opts.files)?;
    if context.corr_version != CorrelatorVersion::V2 {
        bail!("Input data is not MWAX data; exiting.");
    }
    let floats_per_fine_channel = context.num_visibility_pols * 2;

    let mut sum: f64 = 0.0;
    for timestep_index in 0..context.num_timesteps {
        for coarse_channel_index in 0..context.num_coarse_channels {
            let data = context.read_by_baseline(timestep_index, coarse_channel_index)?;
            for baseline in 0..context.num_baselines {
                // We want the first fine chan for each baseline
                let start_index =
                    baseline * (context.num_fine_channels_per_coarse * floats_per_fine_channel);
                let end_index = start_index + floats_per_fine_channel;

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
