// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

/// Given an voltage observation's data, verify that `mwalib` is functioning correctly
/// by printing an observation context.
use anyhow::*;
use structopt::StructOpt;

use mwalib::*;

#[derive(StructOpt, Debug)]
#[structopt(name = "mwalib-print-volt-context", author)]
struct Opt {
    /// The path to an observation's metafits file.
    #[structopt(short, long, parse(from_os_str))]
    metafits: std::path::PathBuf,

    /// Paths to the observation's voltage files.
    #[structopt(name = "VOLTAGE FILE", parse(from_os_str))]
    files: Vec<std::path::PathBuf>,
}

fn main() -> Result<(), anyhow::Error> {
    let opts = Opt::from_args();
    let context = VoltageContext::new(&opts.metafits, &opts.files)?;

    println!("{}", context);

    Ok(())
}
