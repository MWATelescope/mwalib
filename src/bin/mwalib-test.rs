/// Given an observation's data, verifies that mwalib is functioning correctly.

use anyhow::*;
use structopt::StructOpt;

use mwalib::*;

#[derive(StructOpt, Debug)]
#[structopt(name = "mwalib-test", author)]
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
    let context = mwalibObsContext::new(&opts.metafits, &opts.files);
    println!("{}", context?);

    Ok(())
}
