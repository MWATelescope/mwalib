use std::{env, path::PathBuf};

// This code is adapted from pkg-config-rs
// (https://github.com/rust-lang/pkg-config-rs).
fn infer_static(name: &str) -> bool {
    #[allow(clippy::if_same_then_else, clippy::needless_bool)]
    if env::var(format!("{}_STATIC", name.to_uppercase())).is_ok() {
        true
    } else if env::var(format!("{}_DYNAMIC", name.to_uppercase())).is_ok() {
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
    // Gather build time info
    built::write_built_file().expect("Failed to acquire build-time information");

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

    // Only do this if we're not on docs.rs (doesn't like writing files outside
    // of OUT_DIR).
    match env::var("DOCS_RS").as_deref() {
        Ok("1") => (),
        _ => {
            // Re-run if source changes that affect the header
            println!("cargo:rerun-if-changed=src");
            println!("cargo:rerun-if-changed=cbindgen.toml");

            let crate_dir =
                env::var("CARGO_MANIFEST_DIR").expect("build.rs: CARGO_MANIFEST_DIR not set");

            // Emit header to OUT_DIR and (optionally) mirror into ./include for IDEs
            let out_dir = PathBuf::from(env::var("OUT_DIR").expect("build.rs: OUT_DIR not set"));

            let out_header = out_dir.join("mwalib.h");
            let mirror_header = PathBuf::from(&crate_dir).join("include").join("mwalib.h");

            // Prefer cbindgen.toml; only set deltas here if truly dynamic.
            // Example delta: force pragma_once at build time for a specific target.
            let config = cbindgen::Config::from_root_or_default(&crate_dir);

            let bindings = cbindgen::Builder::new()
                .with_crate(&crate_dir)
                .with_config(config)
                .generate()
                .expect("build.rs: Unable to generate C bindings via cbindgen");

            // Write to OUT_DIR
            bindings.write_to_file(&out_header);

            // Mirror into ./include (optional convenience)
            std::fs::create_dir_all(mirror_header.parent().unwrap())
                .expect("build.rs: failed to create include/ directory");
            std::fs::copy(&out_header, &mirror_header)
                .expect("build.rs: failed to copy header into include/");
        }
    }
}
