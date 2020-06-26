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
    match env::var("MWALIB_LINK_STATIC_CFITSIO") {
        Ok(val) => {
            match val.as_str() {
                "0" => {
                    println!("cargo:warning=rustc will link with the shared libcfitsio.so library. Set MWALIB_LINK_STATIC_CFITSIO=1 in your environment to link statically.")    
                },
                _ => {
                    println!("cargo:rustc-link-lib=static=cfitsio");
                    println!("cargo:warning=rustc will link with the static libcfitsio.a library. Remove MWALIB_LINK_STATIC_CFITSIO from your environment (or set to 0) to link dynamically.");
                }
            }            
        },
        Err(_) => println!("cargo:warning=rustc will link with the shared libcfitsio.so library. Set MWALIB_LINK_STATIC_CFITSIO=1 in your environment to link statically."),
    }
}
