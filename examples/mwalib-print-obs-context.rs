// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

/// Given an observation's data, verify that `mwalib` is functioning correctly
/// by printing an observation context.
use anyhow::*;
use structopt::StructOpt;

use mwalib::*;

#[cfg(not(tarpaulin_include))]
#[derive(StructOpt, Debug)]
#[structopt(name = "mwalib-print-obs-context", author)]
struct Opt {
    /// The path to an observation's metafits file.
    #[structopt(short, long, parse(from_os_str))]
    metafits: std::path::PathBuf,

    /// Paths to the observation's gpubox files.
    #[structopt(name = "GPUBOX FILE", parse(from_os_str))]
    files: Vec<std::path::PathBuf>,
}

#[cfg(not(tarpaulin_include))]
fn main() -> Result<(), anyhow::Error> {
    let opts = Opt::from_args();
    let mut context = mwalibContext::new(&opts.metafits, &opts.files)?;

    context.rf_inputs.sort_by_key(|k| k.subfile_order);

    println!("{}", context);

    Ok(())
}
