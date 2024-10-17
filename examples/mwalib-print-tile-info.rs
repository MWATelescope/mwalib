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
    #[clap(short, long)]
    metafits: std::path::PathBuf,
    // /// Paths to the observation's gpubox files.
    // #[clap(name = "GPUBOX FILE", parse(from_os_str))]
    // files: Vec<std::path::PathBuf>,
}

fn main() -> Result<(), anyhow::Error> {
    env_logger::try_init().unwrap_or(());
    let opts = Opt::parse();
    let context = MetafitsContext::new(opts.metafits, None)?;

    println!("idx\tant\tid  \tname   \tpol\trx\tslot\tlength\trx_type\tflavour   \twhitening");
    for (i, input) in context.rf_inputs.iter().enumerate() {
        println!(
            "{i:3}\t{:3}\t{:4}\t{:7}\t{:3}\t{:2}\t{:4}\t{:+7.1}\t{:7}\t{:10}\t{:9}\t",
            input.ant,
            input.tile_id,
            input.tile_name,
            input.pol.to_string(),
            input.rec_number,
            input.rec_slot_number,
            input.electrical_length_m,
            input.rec_type.to_string(),
            input.flavour.to_string(),
            input.has_whitening_filter
        );
    }

    Ok(())
}
