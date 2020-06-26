# Change Log
Changes in each release are listed below.

## 0.3.2 25-June-2020 (Pre-release)
* libmwalib.so now has statically linked libcfitsio library as cfitsio's ABI keeps changing making linking difficult for users.

## 0.3.1 08-June-2020 (Pre-release)
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