// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

/// Given gpubox files, add the contents of their HDUs and report the sum.
use anyhow::*;
use fitsio::FitsFile;
use structopt::StructOpt;

use mwalib::*;

#[derive(StructOpt, Debug)]
#[structopt(name = "mwalib-sum-gpubox-hdus", author)]
struct Opt {
    /// Print the first x floats from HDU 1 of each gpubox file.
    #[structopt(short, long)]
    floats: Option<usize>,

    /// Don't use mwalib - just iterate over the HDUs and add them. The result
    /// might be different because the start/end times of the observation may
    /// not be consistent.
    #[structopt(long)]
    direct: bool,

    /// Path to the metafits file.
    #[structopt(short, long)]
    metafits: Option<String>,

    /// Paths to the gpubox files.
    #[structopt(name = "GPUBOX FILE")]
    files: Vec<String>,
}

fn sum_direct(files: Vec<String>, floats: Option<usize>) -> Result<(), anyhow::Error> {
    let mut sum: f64 = 0.0;
    let mut first_x = "".to_string();
    for gpubox in files {
        println!("Reading {}", gpubox);
        let mut hdu_index = 1;
        let mut s: f64 = 0.0;
        let mut fptr = FitsFile::open(&gpubox)?;
        while let Ok(hdu) = fptr.hdu(hdu_index) {
            let buffer: Vec<f32> = hdu.read_image(&mut fptr)?;
            if hdu_index == 1 {
                if let Some(f) = floats {
                    first_x = format!("{:?}", buffer.iter().take(f).collect::<Vec<&f32>>());
                }
            }

            s += buffer.iter().map(|v| *v as f64).sum::<f64>();
            hdu_index += 1;
        }

        println!("Sum: {}", s);
        if let Some(f) = floats {
            println!("First {} floats: {}", f, first_x);
        }
        println!();
        sum += s;
    }

    println!("Total sum: {}", sum);
    Ok(())
}

fn sum_mwalib(metafits: String, files: Vec<String>) -> Result<(), anyhow::Error> {
    let mut context = mwalibContext::new(&metafits, &files)?;

    let mut sum: f64 = 0.0;
    let mut data = vec![vec![]];
    while !data.is_empty() {
        data = context.read(1)?;
        for scan in &data {
            for gpubox in scan {
                for value in gpubox {
                    sum += *value as f64;
                }
            }
        }
    }

    println!("Total sum: {}", sum);
    Ok(())
}

fn main() -> Result<(), anyhow::Error> {
    let opts = Opt::from_args();
    if opts.direct {
        sum_direct(opts.files, opts.floats)?;
    } else {
        // Ensure we have a metafits file.
        if let Some(m) = opts.metafits {
            sum_mwalib(m, opts.files)?;
        } else {
            bail!("A metafits file is required when using mwalib.")
        }
    }

    Ok(())
}
