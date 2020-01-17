# mwalib
MWA library to read raw visibilities and metadata into a common structure

## Usage
- Populate a `mwalibArgs_s` struct

    This struct contains only information on the metafits file and gpubox files
    associated with an observation. Use the `initialise_args`,
    `set_metafits_filename`, and `add_gpubox_filename` functions (see args.c) to
    populate the struct.

- Convert the `mwalibArgs_s` struct to a `mwaObsContext_s` struct

    Use the `process_args` function (args.c) to do this. This function does a
    bunch of tests while populating the `mwaObsContext_s` struct, to ensure that
    everything passed in looks OK. The `mwaObsContext_s` struct contains much
    more information on the observation, which will be used to actually extract
    raw data from the gpubox files.

## Concepts
- gpubox "batches"

    gpubox batches refers to the different gpubox outputs for the same
    course-band channel. e.g. `1065880128_20131015134830_gpubox01_00.fits`
    belongs to "batch 0", whereas `1065880128_20131015134930_gpubox01_01.fits`
    belongs to "batch 1".

    `mwaObsContext_s` contains:
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

- "GPSTIME", "NINPUTS", "CHANNELS", "NCHANS" within the metafits file.

When `mwsObsContext_s` is being constructed from `mwalibArgs_s`, the following are also checked:

- The presence of "TIME", "MILLITIM", "NAXIS2" within the gpubox files,

- Consistent number of gpubox files in each batch

    i.e. if `1065880128_20131015134830_gpubox01_00.fits` exists, and there are
    two batches, then `1065880128_20131015134930_gpubox01_01.fits` also exists,
    and likewise for all other coarse-band channels.

- Consistent "batch number format"

    i.e. a gpubox file `1065880128_20131015134830_gpubox01_00.fits` can not be
    used alongside another file like `1065880128_20131015134830_gpubox01.fits`.
