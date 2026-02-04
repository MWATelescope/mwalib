extern crate mwalib;

use std::env;

fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().filter_or("RUST_LOG", "info")).init();

    generate_stubs()?;

    anyhow::Ok(())
}

fn generate_stubs() -> anyhow::Result<()> {
    // Generating the stub requires the below env variable to be set for some reason?
    env::set_var("CARGO_MANIFEST_DIR", env::current_dir()?);
    let stub = mwalib::python::stub_info()?;
    stub.generate()?;

    Ok(())
}
