// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

/// Given an voltage observation's data, verify that `mwalib` is functioning correctly
/// by printing an observation context.
use anyhow::*;
use clap::Parser;

use mwalib::*;

#[derive(Parser, Debug)]
#[clap(name = "mwalib-print-volt-context", author)]
struct Opt {
    /// The path to an observation's metafits file.
    #[clap(short, long)]
    metafits: std::path::PathBuf,

    /// Paths to the observation's voltage files.
    #[clap(name = "VOLTAGE FILE")]
    files: Vec<std::path::PathBuf>,
}

fn main() -> Result<(), anyhow::Error> {
    env_logger::try_init().unwrap_or(());
    let opts = Opt::parse();
    let context = VoltageContext::new(opts.metafits, &opts.files)?;

    println!("{}", context);

    Ok(())
}
