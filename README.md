# mwalib
MWA library to read raw visibilities and metadata into a common structure

## Usage via C
See `test.c` and `build.sh` for an example.

## Usage via Python
TODO. Will provide instructions adapted from
http://jakegoulding.com/rust-ffi-omnibus/objects/

## Usage in Rust
See `mwalib-test.rs` for an example.

- Populate a `mwalibObsContext` struct

    This struct contains only information on the metafits file and gpubox files
    associated with an observation. Use the `mwalibObsContext_new` function (see
    test.c for an example) to populate the struct. This function does a bunch of
    tests while populating the `mwalibObsContext` struct, to ensure that
    everything passed in looks OK. The struct unpacks information on the
    observation, which will be used to actually extract raw data from the gpubox
    files.

## Concepts
- gpubox "batches"

    gpubox batches refers to the different gpubox outputs for the same
    course-band channel. e.g. `1065880128_20131015134830_gpubox01_00.fits`
    belongs to "batch 0", whereas `1065880128_20131015134930_gpubox01_01.fits`
    belongs to "batch 1".

    `mwaObsContext` contains:
    - A count of how many batches are present (e.g. The above filenames would
      have this number set to 2)
    - An array `gpubox_filename_batches` and an array
      `gpubox_ptr_batches`. These arrays just hold pointers to the data
      elsewhere in the same struct (specifically `gpubox_filenames` and
      `gpubox_ptrs`) to minimise the footprint of the struct, and avoid
      something crazy like two file pointers to the same
      file. `gpubox_filename_batches` and `gpubox_ptr_batches` are structured
      like e.g. `gpubox_filename_batches[batch][0] = pointer to first filename`

    Old-style gpubox filenames are also handled
    (e.g. `1059244752_20130730183854_gpubox01.fits`).

## Consistency checks
`mwalib` checks input files for the presence of:

- a metafits file,

- gpubox files,

- "GPSTIME", "NINPUTS", "CHANNELS", "NCHANS" within the metafits file,

- "TIME", "MILLITIM", "NAXIS2" within the gpubox files.

When `mwalibObsContext` is being constructed, the following are also checked:

- Consistent number of gpubox files in each batch

    i.e. if `1065880128_20131015134830_gpubox01_00.fits` exists, and there are
    two batches, then `1065880128_20131015134930_gpubox01_01.fits` also exists,
    and likewise for all other coarse-band channels.

- Consistent "batch number format"

    i.e. a gpubox file `1065880128_20131015134830_gpubox01_00.fits` can not be
    used alongside another file like `1065880128_20131015134830_gpubox01.fits`.

## Example test output
```
mwalibObsContext (                                                                                                                                                                                                   
    obsid:               1065880128,                                                                                                                                                                                 
    obs UNIX start time: 1381844910 s,                                                                                                                                                                               
    obs UNIX end time:   1381845018.5 s,                                                                                                                                                                             

    num integrations: 0,
    num baselines:    8128,
    num pols:         4,

    num fine channels: 32,
    coarse channels: [131, 132, 133, 134, 135, 136, 137, 138, 139, 140, 141, 142, 143, 144, 145, 146, 147, 148, 149, 150, 151, 152, 153, 154],
    fine channel resolution:  40 kHz,
    coarse channel bandwidth: 30.72 MHz,

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
