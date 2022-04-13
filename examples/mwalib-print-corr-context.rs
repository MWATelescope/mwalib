// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

// Given an observation's data, verify that `mwalib` is functioning correctly
// by printing an observation context.

// run this example with:
// $ cargo run --example mwalib-print-obs-context -- --metafits metafits_filename gpuboxfilename1 gpuboxfilename2...
//
// Turn on logging with: (then rerun)
// $ export RUST_LOG=mwalib=debug
//
use anyhow::*;
use clap::Parser;

use mwalib::*;

#[derive(Parser, Debug)]
#[clap(name = "mwalib-print-obs-context", author)]
struct Opt {
    /// The path to an observation's metafits file.
    #[clap(short, long, parse(from_os_str))]
    metafits: std::path::PathBuf,

    /// Paths to the observation's gpubox files.
    #[clap(name = "GPUBOX FILE", parse(from_os_str))]
    files: Vec<std::path::PathBuf>,
}

fn main() -> Result<(), anyhow::Error> {
    env_logger::try_init().unwrap_or(());
    let opts = Opt::parse();
    let context = CorrelatorContext::new(opts.metafits, &opts.files)?;

    println!("{}", context);

    Ok(())
}
