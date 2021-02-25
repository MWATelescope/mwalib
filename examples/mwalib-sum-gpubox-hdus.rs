// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

/// Given gpubox files, add the contents of their HDUs and report the sum.
use anyhow::*;
use structopt::StructOpt;

use mwalib::*;

#[cfg(not(tarpaulin_include))]
#[derive(StructOpt, Debug)]
#[structopt(name = "mwalib-sum-gpubox-hdus", author)]
struct Opt {
    /// Don't use mwalib - just iterate over the HDUs and add them. The result
    /// might be different because the start/end times of the observation may
    /// not be consistent.
    #[structopt(long)]
    direct: bool,

    /// Path to the metafits file.
    #[structopt(short, long, parse(from_os_str))]
    metafits: std::path::PathBuf,

    /// Paths to the gpubox files.
    #[structopt(name = "GPUBOX FILE", parse(from_os_str))]
    files: Vec<std::path::PathBuf>,
}

#[cfg(not(tarpaulin_include))]
fn sum_direct(files: Vec<std::path::PathBuf>) -> Result<(), anyhow::Error> {
    println!("Summing directly from HDUs...");
    let mut sum: f64 = 0.0;
    for gpubox in files {
        println!("Reading {}", gpubox.display());
        let mut hdu_index = 1;
        let mut s: f64 = 0.0;
        let mut fptr = fits_open!(&gpubox)?;
        while let Ok(hdu) = fits_open_hdu!(&mut fptr, hdu_index) {
            let buffer: Vec<f32> = get_fits_image!(&mut fptr, &hdu)?;
            s += buffer.iter().map(|v| *v as f64).sum::<f64>();
            hdu_index += 1;
        }

        println!("Sum: {}", s);
        sum += s;
    }

    println!("Total sum: {}", sum);
    Ok(())
}

#[cfg(not(tarpaulin_include))]
fn sum_mwalib<T: AsRef<std::path::Path>>(metafits: &T, files: &[T]) -> Result<(), anyhow::Error> {
    println!("Summing via mwalib using read_by_baseline()...");
    let mut context = CorrelatorContext::new(metafits, files)?;
    println!("Correlator version: {}", context.corr_version);

    let mut sum: f64 = 0.0;
    let mut count: u64 = 0;

    for t in 0..context.num_timesteps {
        for c in 0..context.num_coarse_chans {
            let data = context.read_by_baseline(t, c)?;

            for b in 0..context.metafits_context.num_baselines {
                let baseline_index = b
                    * (context.metafits_context.num_corr_fine_chans_per_coarse
                        * context.metafits_context.num_visibility_pols
                        * 2);

                for f in 0..context.metafits_context.num_corr_fine_chans_per_coarse {
                    let fine_chan_index = f * (context.metafits_context.num_visibility_pols * 2);

                    for v in 0..8 {
                        sum += data[baseline_index + fine_chan_index + v] as f64;
                    }
                    count += 8;
                }
            }
        }
    }

    println!("Sum: {}; Count: {}", sum, count);

    Ok(())
}

#[cfg(not(tarpaulin_include))]
fn main() -> Result<(), anyhow::Error> {
    let opts = Opt::from_args();
    if opts.direct {
        sum_direct(opts.files)?;
    } else {
        sum_mwalib(&opts.metafits, &opts.files)?;
    }

    Ok(())
}
