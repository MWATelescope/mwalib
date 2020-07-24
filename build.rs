use std::env;

fn main() {
    //
    // Link to shared or static CFITSIO
    //
    // Check if we have this environment variable
    // If we do, then we will link to cfitsio statically
    // But if so, you need to have:
    // 1. libcfitsio.a in your LD_LIBRARY_PATH or PATH
    // AND
    // 2. libcfitsio.a needs to have been built with the following ./configure statement:
    //    ./configure --disable-curl --prefix=/usr/local --enable-reentrant
    if let Ok(val) = env::var("MWALIB_LINK_STATIC_CFITSIO") {
        match val.as_str() {
            "0" => (),
            _ => println!("cargo:rustc-link-lib=static=cfitsio"),
        }
    }
}
