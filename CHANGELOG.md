# Change Log

Changes in each release are listed below.

## 1.1.0 13-Sep-2023

* BREAKING CHANGE: `VoltageContext::read_second` and `read_file` now return signed bytes (`i8`) instead of unsigned (`u8`). This is a very breaking change for existing clients using these functions, but it is necessary to return the correct interpretation of the bytes to the client.
* Python classes and methods now have docstrings. 

## 1.0.1 12-Sep-2023

* Fixed various uncaught clippy lints.

## 1.0.0 07-Sep-2023

* Reverted MSRV to 1.63 for compatibility with manylinux standard.
* Python bindings are now available (as well as the existing C bindings). This replaces the `pymwalib` python project. See the `README.md` for help using it.
* Added `rec_type` to the `rf_input` struct so the client can determine the receiver type of the connected receiver.
* Minor linting and documentation fixes.
* Overhaul of CI workflows to produce Python wheels and various levels of optimisation for Linux and MacOSX for the library itself.

## 0.17.0 21-Aug-2023

* Bumped MSRV to 1.64.
* MetafitsContext now supports: OVERSAMP from metafits. Exposed as a bool: `oversampled`. If this is true, then the observation was taken using oversampled coarse channels.
* VoltageContext now supports handling reading of oversampled subfiles.
* MetafitsContext now supports: DR_FLAG and DR_PARAM from metafits. `deripple_applied`is a boolean and `dreipple_param` is a String. If deripple is true then a deripple has been applied to each coarse channel to smooth the passband.
* Weights (packet occupancy) is now available via the CorrelatorContext struct method `read_weights_by_baseline()` and `read_weights_by_baseline_into_buffer()`.

## 0.16.4 16-Jun-2023

* Modified rf_input.vcs_order to return `input` if `input` is > 255

## 0.16.3 09-Jun-2023

* Removed dependence on clap 3 via cbindgen

## 0.16.2 27-Apr-2023

* Updated min rust version to 1.60
* Updated linux tests CI to test on Ubuntu 20.04 and 22.04

## 0.16.1 27-Apr-2023

* MWAX Voltage Buffer mode `MWAX_BUFFER` is now supported.

## 0.16.0 08-Dec-2022

* FFI/C: `MetafitsMetadata`->`ra_phase_center_deg` is now NaN not 0 when missing from metafits file.
* FFI/C: `MetafitsMetadata`->`dec_phase_center_deg` is now NaN not 0 when missing from metafits file.
* `MetafitsContext::grid_name` defaults to 'NOGRID' when key is missing from metafits file.
* `MetafitsContext::grid_number` defaults to 0 when key is missing from metafits file.
* `MetafitsContext::sun_alt_deg` is now an Option&lt;f64&gt; to handle cases where key is missing from metafits file. Via FFI/C it is represented by a NaN.
* `MetafitsContext::sun_distance_deg` is now an Option&lt;f64&gt; to handle cases where key is missing from metafits file. Via FFI/C it is represented by a NaN.
* `MetafitsContext::moon_distance_deg` is now an Option&lt;f64&gt; to handle cases where key is missing from metafits file. Via FFI/C it is represented by a NaN.
* `MetafitsContext::jupiter_distance_deg` is now an Option&lt;f64&gt; to handle cases where key is missing from metafits file. Via FFI/C it is represented by a NaN.
* Update module docs.
* Update dependencies.
* Fixed a bunch of clippy lints.

## 0.15.1 09-Aug-2022

* mwalib now requires Rust version 1.57 or newer
* Speed up conversion of legacy -> MWAX visibility ordering
* Expose all mwalib error types

## 0.15.0 23-Jun-2022

* Provide DUT1 in the metafits context
* API tweaks surrounding generic types
  * Things should be slightly more flexible than before
* Clippy lints and update cbindgen
* Rename mwa_rust_core -> Marlu

## 0.14.0 13-Apr-2022

* Specify minimum-required dependency versions.
* Clean up fits long-string functions.
* Stop borrowing generic types.
  * This applied only to single files (like a metafits file); the required was
    type `&T` (where `T` is `AsRef<Path>`) when `T` alone is sufficient.
* Clippy lints and Rust 2021.
  * Appeased most Clippy lints and upped the Rust edition to 2021. mwalib now
    also specifies the minimum supported Rust version and has CI tests against
    this version.
* Lighten code to get channel frequencies.
* Changed metafits_context `cable_delays_applied` from bool to an enum `CableDelaysApplied`.

## 0.13.1 13-Apr-2022

* Fixed calculation of number of baselines for cases where num_antennas is odd.

## 0.13.0 04-Feb-2022

* rfinput->digital_gains are now f64 and are divided by 64 (the value in the metafits is 64x larger to allow more flexible gains settings)
* Modernised examples using clap 3.0 and reduced dependencies

## 0.12.2 17-Jan-2022

* `get_optional_fits_key` now handles the CFITSIO return code of 204 (when there no value for the key) and correctly returns None instead of an error
* The `digital_gains array` in `rf_input` is now sized based on the `num_metafits_coarse_chans` rather than hardcoded to be 24
* `VoltageContext`'s voltage batches and voltage time map are now (library) public like the equivalent attributes in `CorrelatorContext`
* Updated code coverage github action as it had stopped working

## 0.12.1 29-Nov-2021

* Fixed issue and covered with tests the case where mwalib would panic when creating `CorrelatorContext` with only gpubox file(s) from the second or higher batch (e.g. 001,002...).

## 0.12.0 12-Nov-2021

* Added calibrator and calibrator_source to `metafits_context`
* Reverted efa8ca41edccbe15079642b26ed5049a8656e3a9 (behaviour of coarse_chan.gpubox_number for VoltageContext LegacyVCS use-case)

## 0.11.0 28-Oct-2021

* Added new metafits key RAWSCALE to metafits context
* Made gridnum and global_analogue_attenuation_db optional (since they are not in every historical metafits)
* For Legacy VCS, coarse channel.gpuboxnumber now matches the values used by the Legacy Correlator (instead of receiver channel number)
* Added back constants recently removed as they were needed by FFI users: MWALIB_MWA_LATITUDE_RADIANS, MWALIB_MWA_LONGITUDE_RADIANS, MWALIB_MWA_ALTITUDE_METRES, MWALIB_SPEED_OF_LIGHT_IN_VACUUM_M_PER_S. The constants are prefixed with "MWALIB_" to ensure no clashes with mwa_rust_core constants
* Fixed missing nul terminators on returned strings in mwalib FFI functions

## 0.10.0 20-Aug-2021

* Fixed bug where the `num_metafits_fine_chan_freqs` and `metafits_fine_chan_freqs_hz` were not being populated correctly for a stand-alone `MetafitsContext` usecase.
* Fixed attribute name `metafits_fine_chan_freqs` to `metafits_fine_chan_freqs_hz` in FFI `MetafitsMetadata`to be consistent with the rest of the library.
* Removed constants `SPEED_OF_LIGHT_IN_VACUUM_M_PER_S`, `MWA_LATITUDE_RADIANS`, `MWA_LONGITUDE_RADIANS`, `MWA_ALTITUDE_METRES` as these are now provided by the [mwa_rust_core](https://github.com/MWATelescope/mwa_rust_core) repo.
* Simplified the dependency rules for `regex` and `rayon` crates.

## 0.9.4 19-Aug-2021

## 0.9.3 10-Aug-2021

* Antenna rf_input_x and rf_input_y are now correctly indexed/sorted for the VCSLegacyRecombined case.

## 0.9.2 09-Aug-2021

* When using a stand-alone MetafitsContext, the rf_inputs are now correctly sorted for the VCSLegacyRecombined case.

## 0.9.1 09-Aug-2021

* Added alternative version of mwalib_metafits_context_new (mwalib_metafits_context_new2) to FFI interface which does not require an MWAVersion and will determine it via the MODE keyword.
* Fixed errors, ommissions in comment/documentation for FFI function mwalib_metafits_get_expected_volt_filename().

## 0.9.0 09-Aug-2021

* Added mwa_version &lt;Option&lt;MWAVersion&gt;&gt; to MetafitsContext struct.
* When working only with a MetafitsContext, a None can be passed in lieu of an MWAVersion, and mwalib will attempt to determine the correct MWAVersion based on the MODE keyword from the metafits file.
* Added method get_expected_volt_filename() function to MetafitsContext.
* Added digital_gains, dipole_gains and dipole_delays to FFI rfinput struct.
* Added num_receivers, num_delays to MetafitsContext / metafits_metadata struct in FFI.
* Metafits metadata sched_start_utc and sched_end_utc are now typed as time_t and represent the Unix timestamp (which can be used by various time.h functions).

## 0.8.7 03-Aug-2021

* Updated some dependencies in Cargo.toml.
* Renamed metafits_context.metafits_fine_chan_freqs by adding _hz suffix to be consistent with other naming of attributes with units.

## 0.8.6 02-Aug-2021

* Fixed bug where the metafits.num_metafits_fine_chan_freqs was not being set correctly.

## 0.8.5 02-Aug-2021

* Added helper function get_fine_chan_freqs_hz_array to correlator context and voltage context.
* Added metafits_context.num_metafits_fine_chan_freqs & metafits_context.metafits_fine_chan_freqs, providing a vector of sky frequencies for all fine channels.
* Added metafits_context.volt_fine_chan_width_hz & metafits_context.num_volt_fine_chans_per_coarse to describe the voltage fine channel configuration.
* Added the above new functions and attributes to equivalent structs in FFI.
* Added more badges to github README.

## 0.8.4 15-Jul-2021

* mwalib legacy autocorrelations (where ant1==ant2) are now conjugated with respect to previous versions.

## 0.8.3 01-Jul-2021

* Refactor of FFI: collapsing, antennas, baselines, coarse channels, rf inputs and timesteps into attributes of the MetafitsMetadata, CorrelatorMetadata and Voltage Metadata structs.
* Refactor of FFI: Added common, common good and provided timesteps/coarse channels to CorrelatorMetadata and VoltageMetadata structs.
* Refactor of FFI: Added delays and recievers to MetafitsMetadata.
* Added const for speed of light in a vacuum.
* Detected and raise error condition when gpubox fits file has no data.
* Updated VoltageContext display to be consistent with CorrelatorContext.
* Fixed bugs when reading metafits files which have new MWAX GEODEL, CALIBDEL and CABLEDEL keys.

## 0.8.2 09-Jun-2021

* Added common, common good and provided timesteps/coarse channels for VoltageContext.
* Voltage timesteps and coarse channel vectors now represent the superset of metafits and provided data.
* Due to the above, the read methods now may return a NoDataForTimeStepCoarseChannel error if the timestep/coarse channel combination does not have a file available to read data from.

## 0.8.1 09-Jun-2021

* Modified logic of common and good correlator timesteps/coarse channels to mean common to all provided coarse channels.
* Added provided_coarse_chans which is a vector of all of the provided gpubox files coarse channels.

## 0.8.0 08-Jun-2021

* Removed VisibilityPol struct, replaced with VisPol enum to simplify.
* Renamed CorrelatorVersion enum to MWAVersion. This now incorporates Correlator version (OldLegacy, Legacy and MWAXv2, as well as VCSRecombined and MWAXVCS).
* MetafitsContext struct now requires an extra parameter: mwa_version in order to correctlty interpret the metadata when only a metafits file is supplied (and no data files).
* Renamed COAX_V_FACTOR to MWA_COAX_V_FACTOR to bring it in line with other mwalib consts.
* Added metafits_coarse_chans to MetafitsContext which represent all the coarse channels that the metafits file defines.
* Added metafits_timesteps to MetafitsContext which represent all the timesteps that the metafits file defines.
* Added geometric_delays_applied enum to MetafitsContext & FFI to inform the caller if and what type of geometric delays have been applied to the data.
* Added cable_delays_applied bool to MetafitsContext & FFI to inform the caller if cable length corrections have been made to the data.
* Added calibration_delays_and_gains_applied bool to MetafitsContext & FFI to inform the caller if calibration delays and gains have been applied to the data.
* Added mode enum to MetafitsContext & FFI to inform the caller of the mode of the observation.
* Added electrical_length_m, north_m, east_m, height_m to the Antenna struct (for efficiency). These are also available in the Rfinput struct.
* Added common timesteps and coarse channels which represent the timesteps and channels common the most max set of coarse channels provided.
* Added common good timesteps and coarse channels which is the same as common, except only timesteps after the quack time are used to determine them.

## 0.7.1 15-May-2021

* Implemented _into_buffer variant of read_by_baseline and read_by_frequency so you can allocate your own vector and pass the whole thing or a slice to be filled with the relevant data.
* The implementation of the ffi read_by_baseline and read_by_freqeuency now use this method, thus reducing an unneeded allocation and copy.
* Added built crate to allow rust callers to query the mwalib build environment (including the mwalib version number).
* Exposed major, minor and patch version of mwalib to ffi callers.
* Pinned rust nightly build in coverage CI to prevent a missing crate error (temporarily?)

## 0.7.0 30-Apr-2021

* Added support for reading voltage data by file or gps second.
* Added FFI support for reading voltage data by file or gps second.
* Removed num_samples_per_timestep from VoltageContext. Added more useful struct members describing the data shape precisely.
* Removed unneeded muts from correlator and voltage contexts.
* Minor cleanup of rust examples.

## 0.6.3 28-Mar-2021

* Refactored github actions for a more complete CI workflow with automated releases.
* Updated README and install instructions and fixed many markdown issues.

## 0.6.2 25-Mar-2021

* Modified MWA Legacy read code to produce cotter-compatible visibilities:
  * mwalib differs from cotter: cotter produces 0+0j for XY on auto's, mwalib provides the values.
  * mwalib and cotter differ from pyuvdata: mwalib/cotter visibilities are conjugated with respect to pyuvdata for cross correlations.
* Added cotter validation data and test to ensure conversion code produces cotter equivalent visibilities (with the above exception).
* Provide rust-fitsio's cfitsio-static feature.
* Bumped fitsio dependency to 0.17.* and  changed fitsio-sys to ^0 to ensure fitsio and mwalib use the same fitsio-sys version always.
* Updated CI pipeline: bumped install of cfitsio to v3.49.
* Made error handling code a little lighter.

## 0.6.1 08-Mar-2021

* Fixed za (zenith angle) calculation.
* Added more comprehensive testing for some coarse_channel methods.
* Addressed many clippy lints.

## 0.6.0 05-Mar-2021

* Minor updates to enable packaging and deployment to crates.io.
* Fixed visibility of library structs and functions.
* Moved coax_v_factor, mwa_lat_radians, mwa_long_radians and mwa_alt_meters out of metafits_metadata and are now just library constants.

## 0.5.1 04-Mar-2021

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

## 0.4.4 08-Jan-2021

* Added receiver_number and receiver_slot_number to rfinput struct.

## 0.4.3 09-Nov-2020

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

## 0.3.2 25-Jun-2020

* libmwalib.so now has statically linked libcfitsio library as cfitsio's ABI keeps changing making linking difficult for users.

## 0.3.1 08-Jun-2020

* Bugfix: Fixed panic when all 24 coarse channels are using receiver channel numbers >128.
* Added more inline documentation for mwalib.h.
* Improved the output, by making it more complete, when displaying the contents of the context object.

## 0.3.0 14-May-2020

* Added baseline array.
* Added visibility pol(arisations) array.
* Added extra fields for scheduled end MJD, UTC and Unix times.
* Added extra validation for timestamps not common to all gpubox files.
* Added extra validation to ensure NAXIS1 and NAXIS2 across all gpubox files match expected values from metafits.
* Metadata reading from gpubox files is now done in parallel.

## 0.2.0 20-Mar-2020

* Initial pre-release.
