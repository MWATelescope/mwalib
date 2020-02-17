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

fn main() -> Result<(), anyhow::Error> {
    let opts = Opt::from_args();

    let mut context = mwalibContext::new(&opts.metafits, &opts.files)?;
    if context.corr_version != CorrelatorVersion::V2 {
        bail!("Input data is not MWAX data; exiting.");
    }

    context.num_data_scans = 3;

    let mut sum: f64 = 0.0;
    while context.num_data_scans != 0 {
        for scan in context.read(context.num_data_scans)?.into_iter() {
            for gpubox in scan {
                for bl in 0..context.num_baselines {
                    for pol in 0..4 {
                        let index = bl * context.num_fine_channels * context.num_antenna_pols * 2 + pol * 2;
                        sum += gpubox[index] as f64;
                        sum += gpubox[index + 1] as f64;
                    }
                }
            }
        }
    }

    println!("Sum of every first fine channel: {} == {:+e}", sum, sum);

    Ok(())
}
