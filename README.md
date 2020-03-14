# mwalib
![Run Tests](https://github.com/MWATelescope/mwalib/workflows/Run%20Tests/badge.svg)
MWA library to read raw visibilities and metadata into a common structure. 
mwalib supports the existing "legacy" MWA correlator, as well as the in-development
"MWAX" correlator. This library strives to provide a single interface to work will 
all incarnations of MWA correlator formats and abstract away the nitty gritty details
about reading MWA data.

## Usage via C
In the `examples` directory, see `build.sh`, `mwalib-print-obs-context.c`, and
`mwalib-sum-all-hdus.c` for examples of usage.

## Usage via Python
In the `examples` directory, see `build.sh`, `mwalib-print-obs-context.py`, and
`mwalib-sum-all-hdus.py` for examples of usage.

## Usage in Rust
See `mwalib-print-obs-context.rs` and `mwalib-print-obs-context.rs` for
examples. Also run `cargo doc --open` to see the rendered documentation.

## Usage approach
- Populate a `mwalibContext` struct

    This struct contains only information on the metafits file and gpubox files
    associated with an observation. During creation, a number of checks are
    performed on the MWA data to ensure consistency. Once created, the
    observation is primed for reading data.

- Read raw data

    The `read_by_baseline` function associated with `mwalibContext` takes in a
    timestep index (see: context.timesteps vector) and a coarse channel index
    (see: context.coarse_channels vector) and will return a vector of 32bit
    floats. The data is organised as [baseline][fine_channel][polarisation][r][i].

## Concepts
- gpubox "batches"

    gpubox batches refers to the different gpubox outputs for the same
    course-band channel. e.g. `1065880128_20131015134830_gpubox01_00.fits`
    belongs to "batch 0", whereas `1065880128_20131015134930_gpubox01_01.fits`
    belongs to "batch 1".

- "scans"

    A scan is the raw data from all gpubox files for a single time
    integration.

- baselines

    A baseline is the distance between any two antennas. However, raw MWA data
    also contains auto-correlation data, and so the number of "baselines"
    presented in the data is calculated as `n/2 * (n+1)` (where `n` is the
    number of antennas).

## Installation
It is possible that a dynamic-shared and/or static objects can be provided on
GitHub in the future, but for now, `mwalib` should be compiled from source.

- Install rust

    `https://www.rust-lang.org/tools/install`

- Compile the source

    `cargo build --release`

- Use the dynamic-shared and/or static objects in the `target/release` directory

    e.g. on linux, `libmwalib.a` or `libmwalib.so`

    These have silly names because of how C historically links libraries. It's
    overall clearer to link "mwalib" than "mwa", so it is left like this.

    For an example of using `mwalib` with C or python, look in the `examples`
    directory.

- (Optional) If you are modifying the ffi functions, you'll need to regenerate mwalib.h

    `cargo install cbindgen`
        

## Consistency checks
`mwalib` checks input files for the presence of:

- a metafits file,

- gpubox files,

- "GPSTIME", "NINPUTS", "CHANNELS", "NCHANS" within the metafits file,

- "TIME", "MILLITIM", "NAXIS2" within the gpubox files.

- Consistent number of gpubox files in each batch

    i.e. if `1065880128_20131015134830_gpubox01_00.fits` exists, and there are
    two batches, then `1065880128_20131015134930_gpubox01_01.fits` also exists,
    and likewise for all other coarse-band channels.

- Consistent "batch number format"

    i.e. a gpubox file `1065880128_20131015134830_gpubox01_00.fits` can not be
    used alongside another file like `1065880128_20131015134830_gpubox01.fits`.

## Example test output
```
mwalibContext (
    Correlator version:  Legacy,

    obsid:               1065880128,
    obs UNIX start time: 1381844910 s,
    obs UNIX end time:   1381845018.5 s,

    num antennas:           128,
    num baselines:          8256,
    num auto-correlations:  128,
    num cross-correlations: 8128,

    num pols:               4,
    num fine channels:      32,
    coarse channels: [131, 132, 133, 134, 135, 136, 137, 138, 139, 140, 141, 142, 143, 144, 145, 146, 147, 148, 149, 150, 151, 152, 153, 154],

    fine channel resolution:  40 kHz,
    coarse channel bandwidth: 30.72 MHz,

    gpubox HDU size:       8.0625 MiB,
    Memory usage per scan: 193.5 MiB,

    metafits filename: /home/chj/WORKING_DIR/MWA/1065880128/1065880128.metafits,
    gpubox batches: [
    [
        "/home/chj/WORKING_DIR/MWA/1065880128/1065880128_20131015134830_gpubox01_00.fits",
        "/home/chj/WORKING_DIR/MWA/1065880128/1065880128_20131015134830_gpubox02_00.fits",
        "/home/chj/WORKING_DIR/MWA/1065880128/1065880128_20131015134830_gpubox03_00.fits",
        "/home/chj/WORKING_DIR/MWA/1065880128/1065880128_20131015134830_gpubox04_00.fits",
        "/home/chj/WORKING_DIR/MWA/1065880128/1065880128_20131015134830_gpubox05_00.fits",
        "/home/chj/WORKING_DIR/MWA/1065880128/1065880128_20131015134830_gpubox06_00.fits",
        "/home/chj/WORKING_DIR/MWA/1065880128/1065880128_20131015134830_gpubox07_00.fits",
        "/home/chj/WORKING_DIR/MWA/1065880128/1065880128_20131015134830_gpubox08_00.fits",
        "/home/chj/WORKING_DIR/MWA/1065880128/1065880128_20131015134830_gpubox09_00.fits",
        "/home/chj/WORKING_DIR/MWA/1065880128/1065880128_20131015134830_gpubox10_00.fits",
        "/home/chj/WORKING_DIR/MWA/1065880128/1065880128_20131015134830_gpubox11_00.fits",
        "/home/chj/WORKING_DIR/MWA/1065880128/1065880128_20131015134830_gpubox12_00.fits",
        "/home/chj/WORKING_DIR/MWA/1065880128/1065880128_20131015134830_gpubox13_00.fits",
        "/home/chj/WORKING_DIR/MWA/1065880128/1065880128_20131015134830_gpubox14_00.fits",
        "/home/chj/WORKING_DIR/MWA/1065880128/1065880128_20131015134830_gpubox15_00.fits",
        "/home/chj/WORKING_DIR/MWA/1065880128/1065880128_20131015134830_gpubox16_00.fits",
        "/home/chj/WORKING_DIR/MWA/1065880128/1065880128_20131015134830_gpubox17_00.fits",
        "/home/chj/WORKING_DIR/MWA/1065880128/1065880128_20131015134830_gpubox18_00.fits",
        "/home/chj/WORKING_DIR/MWA/1065880128/1065880128_20131015134830_gpubox19_00.fits",
        "/home/chj/WORKING_DIR/MWA/1065880128/1065880128_20131015134830_gpubox20_00.fits",
        "/home/chj/WORKING_DIR/MWA/1065880128/1065880128_20131015134830_gpubox21_00.fits",
        "/home/chj/WORKING_DIR/MWA/1065880128/1065880128_20131015134830_gpubox22_00.fits",
        "/home/chj/WORKING_DIR/MWA/1065880128/1065880128_20131015134830_gpubox23_00.fits",
        "/home/chj/WORKING_DIR/MWA/1065880128/1065880128_20131015134830_gpubox24_00.fits",
    ],
    [
        "/home/chj/WORKING_DIR/MWA/1065880128/1065880128_20131015134930_gpubox01_01.fits",
        "/home/chj/WORKING_DIR/MWA/1065880128/1065880128_20131015134930_gpubox02_01.fits",
        "/home/chj/WORKING_DIR/MWA/1065880128/1065880128_20131015134930_gpubox03_01.fits",
        "/home/chj/WORKING_DIR/MWA/1065880128/1065880128_20131015134930_gpubox04_01.fits",
        "/home/chj/WORKING_DIR/MWA/1065880128/1065880128_20131015134930_gpubox05_01.fits",
        "/home/chj/WORKING_DIR/MWA/1065880128/1065880128_20131015134930_gpubox06_01.fits",
        "/home/chj/WORKING_DIR/MWA/1065880128/1065880128_20131015134930_gpubox07_01.fits",
        "/home/chj/WORKING_DIR/MWA/1065880128/1065880128_20131015134930_gpubox08_01.fits",
        "/home/chj/WORKING_DIR/MWA/1065880128/1065880128_20131015134930_gpubox09_01.fits",
        "/home/chj/WORKING_DIR/MWA/1065880128/1065880128_20131015134930_gpubox10_01.fits",
        "/home/chj/WORKING_DIR/MWA/1065880128/1065880128_20131015134930_gpubox11_01.fits",
        "/home/chj/WORKING_DIR/MWA/1065880128/1065880128_20131015134930_gpubox12_01.fits",
        "/home/chj/WORKING_DIR/MWA/1065880128/1065880128_20131015134930_gpubox13_01.fits",
        "/home/chj/WORKING_DIR/MWA/1065880128/1065880128_20131015134930_gpubox14_01.fits",
        "/home/chj/WORKING_DIR/MWA/1065880128/1065880128_20131015134930_gpubox15_01.fits",
        "/home/chj/WORKING_DIR/MWA/1065880128/1065880128_20131015134930_gpubox16_01.fits",
        "/home/chj/WORKING_DIR/MWA/1065880128/1065880128_20131015134930_gpubox17_01.fits",
        "/home/chj/WORKING_DIR/MWA/1065880128/1065880128_20131015134930_gpubox18_01.fits",
        "/home/chj/WORKING_DIR/MWA/1065880128/1065880128_20131015134930_gpubox19_01.fits",
        "/home/chj/WORKING_DIR/MWA/1065880128/1065880128_20131015134930_gpubox20_01.fits",
        "/home/chj/WORKING_DIR/MWA/1065880128/1065880128_20131015134930_gpubox21_01.fits",
        "/home/chj/WORKING_DIR/MWA/1065880128/1065880128_20131015134930_gpubox22_01.fits",
        "/home/chj/WORKING_DIR/MWA/1065880128/1065880128_20131015134930_gpubox23_01.fits",
        "/home/chj/WORKING_DIR/MWA/1065880128/1065880128_20131015134930_gpubox24_01.fits",
    ],
],
)
```
