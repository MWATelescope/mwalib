# Change Log

Changes in each release are listed below.

## 0.6.2 25-Mar-2021 (Pre-release)

* Modified MWA Legacy read code to produce cotter-compatible visibilities:
 * mwalib differs from cotter: cotter produces 0+0j for XY on auto's, mwalib provides the values.
 * mwalib and cotter differ from pyuvdata: mwalib/cotter visibilities are conjugated with respect to pyuvdata for cross correlations.
* Added cotter validation data and test to ensure conversion code produces cotter equivalent visibilities (with the above exception).
* Provide rust-fitsio's cfitsio-static feature. 
* Bumped fitsio dependency to 0.17.* and  changed fitsio-sys to ^0 to ensure fitsio and mwalib use the same fitsio-sys version always.
* Updated CI pipeline: bumped install of cfitsio to v3.49.
* Made error handling code a little lighter. 

## 0.6.1 08-Mar-2021 (Pre-release)

* Fixed za (zenith angle) calculation.
* Added more comprehensive testing for some coarse_channel methods.
* Addressed many clippy lints.

## 0.6.0 05-Mar-2021 (Pre-release)

* Minor updates to enable packaging and deployment to crates.io.
* Fixed visibility of library structs and functions.
* Moved coax_v_factor, mwa_lat_radians, mwa_long_radians and mwa_alt_meters out of metafits_metadata and are now just library constants.

## 0.5.1 04-Mar-2021 (Pre-release)

* Major refactoring- this will break compatibility with previous mwalib versions.
* mwalibContext top level object now split into:
    * MetafitsContext (when you only provide a metafits file)
    * CorrelatorContext (when you provide a metafits and 1 or more gpubox files)
    * VoltageContext (when you provide a metafits and 1 or more voltage files)
* FFI interfaces standardised, with struct based functions returning arrays of structs e.g. mwalib_antennas_get returns an array of antennas rather than a single instance.
* Souce code is broken out into seperate folders with their own test.rs unit tests.
* TimeStep struct now has GPS time as well as UNIX time.
* Many new struct members added.
* Many long named members renamed to use shorter or abbreviated names.
* NOTE: VoltageContext does not have data reading functions in this release (however, metadata is supported). This will be added in an upcoming release.

## 0.4.4 08-Jan-2021 (Pre-release)

* Added receiver_number and receiver_slot_number to rfinput struct.

## 0.4.3 09-Nov-2020 (Pre-release)

* Expose the fitsio and fitsio-sys crates used: This allows callers to use whatever version of fitsio and fitsio-sys that is used by mwalib, in turn ensuring that other dependent libraries aren't using different versions of these crates. And, along with the other change to this crate, means that statically-linking cfitsio from other crates is simpler.
* The `infer_static` function introduced to build.rs is a workaround for pkg-config-rs being too restrictive when static linking (rust-lang/pkg-config-rs#102). Basically, if we decide to statically link, we emit a message to cargo, and it'll work. Hopefully this hack can be removed in the future when pkg-config-rs is a little more liberal.
* No longer keep fits files open unless we are actually reading them. Fixes #7.
* Expose MWA coordinates as library constants.
* Added the reading of FREQCENT key from the metafits file to mwalibContext.
* Sort the order of the struct members going into a new mwalibContext.
* Fix a bunch of clippy lints. Also fix up the benchmarking code.
* Allow passing no gpubox files (i.e. mwalib will read only the metafits file) when creating an mwalibContext instance.
* Read digital gains and dipole delays from metafits. (Currently unavailable via FFI).
* Specify Rfinput polarisation as an enum instead of a string.
* Overhaul the error handling in mwalib, and change the API traits.
* Fixed cargo tarpaulin decorators to use new format.

## 0.3.2 25-Jun-2020 (Pre-release)

* libmwalib.so now has statically linked libcfitsio library as cfitsio's ABI keeps changing making linking difficult for users.

## 0.3.1 08-Jun-2020 (Pre-release)

* Bugfix: Fixed panic when all 24 coarse channels are using receiver channel numbers >128.
* Added more inline documentation for mwalib.h.
* Improved the output, by making it more complete, when displaying the contents of the context object.

## 0.3.0 14-May-2020 (Pre-release)

* Added baseline array.
* Added visibility pol(arisations) array.
* Added extra fields for scheduled end MJD, UTC and Unix times.
* Added extra validation for timestamps not common to all gpubox files.
* Added extra validation to ensure NAXIS1 and NAXIS2 across all gpubox files match expected values from metafits.
* Metadata reading from gpubox files is now done in parallel.

## 0.2.0 20-Mar-2020 (Pre-release)

* Initial pre-release.
