// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

/// Given an observation's data, verify that `mwalib` is functioning correctly
/// by printing an observation context, as well as the sum of the first scan.
use anyhow::*;
use structopt::StructOpt;

use mwalib::*;

#[derive(StructOpt, Debug)]
#[structopt(name = "mwalib-print-obs-context", author)]
struct Opt {
    /// The path to an observation's metafits file.
    #[structopt(short, long)]
    metafits: String,

    /// Paths to the observation's gpubox files.
    #[structopt(name = "GPUBOX FILE")]
    files: Vec<String>,
}

fn main() -> Result<(), anyhow::Error> {
    let opts = Opt::from_args();
    let context = mwalibContext::new(&opts.metafits, &opts.files)?;
    println!("{}", context);

    Ok(())
}
