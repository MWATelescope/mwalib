use std::env;

// This code is adapted from pkg-config-rs
// (https://github.com/rust-lang/pkg-config-rs).
fn infer_static(name: &str) -> bool {
    #[allow(clippy::if_same_then_else, clippy::needless_bool)]
    if env::var(&format!("{}_STATIC", name.to_uppercase())).is_ok() {
        true
    } else if env::var(&format!("{}_DYNAMIC", name.to_uppercase())).is_ok() {
        false
    } else if env::var("PKG_CONFIG_ALL_STATIC").is_ok() {
        true
    } else if env::var("PKG_CONFIG_ALL_DYNAMIC").is_ok() {
        false
    } else {
        false
    }
}

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
    if env::var("MWALIB_LINK_STATIC_CFITSIO") == Ok("1".to_string()) || infer_static("cfitsio") {
        println!("cargo:rustc-link-lib=static=cfitsio");
    }

    // Generate a C header for mwalib and write it to the include directory.
    // This routine only need to be done if the ffi module has changed.
    println!("cargo:rerun-if-changed=src/ffi/mod.rs");
    // Only do this if we're not on docs.rs (doesn't like writing files outside
    // of OUT_DIR).
    match env::var("DOCS_RS").as_deref() {
        Ok("1") => (),
        _ => {
            cbindgen::Builder::new()
                .with_config(cbindgen::Config {
                    cpp_compat: true,
                    pragma_once: true,
                    ..Default::default()
                })
                .with_crate(env::var("CARGO_MANIFEST_DIR").unwrap())
                .with_language(cbindgen::Language::C)
                .generate()
                .expect("Unable to generate bindings")
                .write_to_file("include/mwalib.h");
        }
    }
}
