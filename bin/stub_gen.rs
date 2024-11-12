extern crate mwalib;

#[cfg(feature = "python")]
use std::env;
#[cfg(feature = "python")]
use std::fs::File;
#[cfg(feature = "python")]
use std::io::{Read, Write};
#[cfg(feature = "python")]
use std::path::Path;

#[cfg(test)]
#[cfg(feature = "python")]
use tempdir::TempDir;

#[cfg(feature = "python")]
fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().filter_or("RUST_LOG", "info")).init();

    generate_stubs()?;

    fix_stubs()?;

    anyhow::Ok(())
}

#[cfg(feature = "python")]
fn generate_stubs() -> anyhow::Result<()> {
    // Generating the stub requires the below env variable to be set for some reason?
    env::set_var("CARGO_MANIFEST_DIR", env::current_dir()?);
    let stub = mwalib::python::stub_info()?;
    stub.generate()?;

    Ok(())
}

#[cfg(feature = "python")]
fn fix_stubs() -> anyhow::Result<()> {
    // After the stub is generated, we have some "manual" fixes to do
    let stubfile = String::from("mwalib.pyi");

    // Replace the constructors as pyo3_stub_gen seems to ignore the text_signature
    replace_stub(&stubfile,"def __new__(cls,metafits_filename,mwa_version = ...): ...","def __new__(cls, metafits_filename: str, mwa_version: typing.Optional[MWAVersion]=None)->MetafitsContext:\n        ...\n",)?;

    replace_stub(&stubfile, "def __new__(cls,metafits_filename,gpubox_filenames): ...", "def __new__(cls, metafits_filename: str, gpubox_filenames: list[str])->CorrelatorContext:\n        ...\n")?;

    replace_stub(&stubfile,"def __new__(cls,metafits_filename,voltage_filenames): ...","def __new__(cls, metafits_filename: str, voltage_filenames: list[str])->VoltageContext:\n        ...\n",)?;

    replace_stub(
        &stubfile,
        "def __enter__(self, slf:MetafitsContext) -> MetafitsContext:",
        "def __enter__(self) -> MetafitsContext:",
    )?;

    replace_stub(
        &stubfile,
        "def __enter__(self, slf:CorrelatorContext) -> CorrelatorContext:",
        "def __enter__(self) -> CorrelatorContext:",
    )?;

    replace_stub(
        &stubfile,
        "def __enter__(self, slf:VoltageContext) -> VoltageContext:",
        "def __enter__(self) -> VoltageContext:",
    )?;

    Ok(())
}

/// Inserts new items in the stubfile for when pyo3 cant do it
/// properly.
///
/// Currently this function is not used, but kept in case it is needed later!
///
/// This will:
/// * Open the created stubfile
/// * Find the line in `string_to_find` (fails if not found)
/// * Add a newline and `string_to_add_below` (fails if cannot)
///
/// # Arguments
///
/// * `stubfile` - `Path` representing filename of the stubfile to edit
///
/// * `string_to_find` - string to find, so we know where in the file to make the insert
///
/// * `string_to_add_below` - string to add on a new line below `string_to_find`
///
///
/// # Returns
///
/// * Result Ok if stub file was modified successfully.
///
///
#[allow(dead_code)]
#[cfg(feature = "python")]
fn insert_stub_below<P: AsRef<Path>>(
    stubfile: P,
    string_to_find: &str,
    string_to_add_below: &str,
) -> anyhow::Result<()> {
    // Open and read the file entirely
    let mut src = File::open(stubfile.as_ref())?;
    let mut data = String::new();
    src.read_to_string(&mut data)?;
    drop(src); // Close the file early

    // Run the replace operation in memory
    let mut new_string: String = string_to_find.to_owned();
    new_string.push_str(string_to_add_below);
    let new_data = data.replace(string_to_find, &new_string);

    // Recreate the file and dump the processed contents to it
    let mut dst = File::create(stubfile.as_ref())?;
    dst.write_all(new_data.as_bytes())?;

    anyhow::Ok(())
}

/// Replaces items in the stubfile for when pyo3 cant do it
/// properly.
///
/// This will:
/// * Open the created stubfile
/// * Find the line in `string_to_find` (fails if not found)
/// * Replace it with `string_to_replace` (fails if cannot)
///
/// # Arguments
///
/// * `stubfile` - `Path` representing filename of the stubfile to edit
///
/// * `string_to_find` - string to find (which will be replaced)
///
/// * `string_to_replace` - string to put in `string_to_find`s place
///
///
/// # Returns
///
/// * Result Ok if stub file was modified successfully.
///
///
#[cfg(feature = "python")]
fn replace_stub<P: AsRef<Path>>(
    stubfile: P,
    string_to_find: &str,
    string_to_replace: &str,
) -> anyhow::Result<()> {
    // Open and read the file entirely
    let mut src = File::open(stubfile.as_ref())?;
    let mut data = String::new();
    src.read_to_string(&mut data)?;
    drop(src); // Close the file early

    // Run the replace operation in memory
    let new_data = data.replace(string_to_find, string_to_replace);

    // Recreate the file and dump the processed contents to it
    let mut dst = File::create(stubfile.as_ref())?;
    dst.write_all(new_data.as_bytes())?;

    anyhow::Ok(())
}

#[test]
#[cfg(feature = "python")]
fn test_insert_stub_below() {
    // Create ephemeral temp directory which will be deleted at end of test
    let dir =
        TempDir::new("test_insert_stub_below").expect("Cannot create temp directory for test");

    // Create a test file
    let text_filename = dir.path().join("test.pyi");

    let mut test_file = File::create(&text_filename)
        .unwrap_or_else(|_| panic!("Could not open {}", text_filename.to_str().unwrap()));
    // Write some lines
    let content = "    Hello\n    World\n    1234";
    let _ = test_file.write_all(content.as_bytes());

    // Now run the add_stub command
    insert_stub_below(&text_filename, "    World\n", "    added_string\n")
        .unwrap_or_else(|_| panic!("Could not add_stub to {}", text_filename.to_str().unwrap()));

    // Now reread the file
    let test_file = File::open(&text_filename)
        .unwrap_or_else(|_| panic!("Could not open {}", text_filename.to_str().unwrap()));
    let mut lines = Vec::new();

    for line in std::io::read_to_string(test_file).unwrap().lines() {
        lines.push(line.to_string())
    }

    // Remove our temp dir
    dir.close().expect("Failed to close temp directory");

    assert_eq!(lines[0], "    Hello");
    assert_eq!(lines[1], "    World");
    assert_eq!(lines[2], "    added_string");
    assert_eq!(lines[3], "    1234");
}

#[test]
#[cfg(feature = "python")]
fn test_replace_stub() {
    // Create ephemeral temp directory which will be deleted at end of test
    let dir =
        TempDir::new("mwalib_test_replace_stub").expect("Cannot create temp directory for test");

    // Create a test file
    let text_filename = dir.path().join("test.pyi");

    let mut test_file = File::create(&text_filename)
        .unwrap_or_else(|_| panic!("Could not open {}", text_filename.to_str().unwrap()));
    // Write some lines
    let content = "    Hello\n    World\n    1234";
    let _ = test_file.write_all(content.as_bytes());

    // Now run the add_stub command
    replace_stub(
        &text_filename,
        "    World\n",
        "    This is the replaced string\n",
    )
    .unwrap_or_else(|_| panic!("Could not add_stub to {}", text_filename.to_str().unwrap()));

    // Now reread the file
    let test_file = File::open(&text_filename)
        .unwrap_or_else(|_| panic!("Could not open {}", text_filename.to_str().unwrap()));
    let mut lines = Vec::new();

    for line in std::io::read_to_string(test_file).unwrap().lines() {
        lines.push(line.to_string())
    }

    // Remove our temp dir
    dir.close().expect("Failed to close temp directory");

    assert_eq!(lines[0], "    Hello");
    assert_eq!(lines[1], "    This is the replaced string");
    assert_eq!(lines[2], "    1234");
}
