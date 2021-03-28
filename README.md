# mwalib

![Run Tests](https://github.com/MWATelescope/mwalib/workflows/Run%20Tests/badge.svg)
[![codecov](https://codecov.io/gh/MWATelescope/mwalib/branch/master/graph/badge.svg)](https://codecov.io/gh/MWATelescope/mwalib)

MWA library to read raw visibilities, voltages and metadata into a common structure.
mwalib supports the existing "legacy" MWA correlator, as well as the in-development
"MWAX" correlator. This library strives to provide a single interface to work with
all incarnations of MWA metadatam correlator and voltage formats and abstract away
the nitty gritty details about reading MWA data.

## Usage via C

In the `examples` directory, see `build_c_examples.sh`, `mwalib-print-obs-context.c`, and
`mwalib-sum-all-hdus.c` for examples of usage.

## Usage via Python

The primary Python interface to `mwalib` is
[`pymwalib`](https://github.com/MWATelescope/pymwalib).

There are also a couple of simple examples Python scripts here that use mwalib's ffi interface instead; see
`examples/mwalib-print-obs-context.py`, and `examples/mwalib-sum-all-hdus.py`.

## Usage in Rust

In the `examples` directory, see `mwalib-print-obs-context.rs` and
`mwalib-print-obs-context.rs` for examples. Also run `cargo doc --open` to see
the rendered documentation.

## Usage approach

- Populate a Context struct (`MetafitsContext`, `CorrelatorContext` or `VoltageContext`)

    `CorrelatorContext` - Use this when you have a metafits file and one or more gpubox files. This struct provides information about the correlator observation and provides access to a `MetafitsContext` for metadata, as well as methods for reading raw visibility data.

    `MetafitsContext` - an efficient way to get access to most of the observation metadata. Only requires that you pass a valid MWA metafits file.

    `VoltageContext` - Use this when you have a metafits file and one or more recombined (or MWAX) voltage files. This struct provides information about the VCS observation and provides access to a `MetafitsContext` for metadata, and in an upcoming release- methods for reading raw voltage data.

    During creation of a `CorrelatorContext` or `MetafitsContext`, a number of checks are
    performed on the metadata and gpubox/voltage files to ensure consistency.
    Once created, the context is primed for reading data.

- Read raw data

    The `read_by_baseline` function associated with `CorrelatorContext` takes in a
    timestep index (see: `CorrelatorContext.timesteps` vector) and a coarse channel index
    (see: `CorrelatorContext.coarse_channels` vector) and will return a vector of 32bit
    floats. The data is organised as [baseline][fine_channel][polarisation][r][i].

    The `read_by_frequency` function is similar to `read_by_baseline` but outputs
    data in [fine_channel][baseline][polarisation][r][i] order.

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

## Building From Source

You can build mwalib from source:

- Install rust

    `https://www.rust-lang.org/tools/install`

- Install cfitsio
    If using a package manager install the `cfitsio-dev` package.
    If building from source ensure you build it with with `--enable-reentrant` option.
       See: [FITSIO Homepage](https://heasarc.gsfc.nasa.gov/fitsio/)
    Alternatively get the fitsio-sys crate to compile cfitsio for you by adding `--features cfitsio-static` to your cargo build command (below).

- Compile the source

    `cargo build --release`

- Statically-linking cfitsio
    If any of the MWALIB_LINK_STATIC_CFITSIO, STATIC_CFITSIO or PKG_CONFIG_ALL_STATIC environment variables exist and are set to a non-zero value, rustc will statically link libcfitsio. The default is to dynamically link libcfitsio. This is an attempt to give developers the choice of having a static or dynamic link of the fits library to ease the build and deploy process.

- Use the dynamic-shared and/or static objects in the `target/release` directory

    e.g. on linux, `libmwalib.a` or `libmwalib.so`

    These have silly names because of how C historically links libraries. It's
    overall clearer to link "mwalib" than "mwa", so it is left like this.

    For an example of using `mwalib` with C or python, look in the `examples`
    directory.

- (Optional) If modifying the ffi functions, a `cargo build` will automatically regenerate mwalib.h

## Installation

As an alternative to building from source, we produce github releases whenever features or bug fixes are completed as tarballs. In the release you will find everything you need to use mwalib from C/C++/Python or any other language that can utilise shared libraries:

- lib/libmwalib.a      (Statically compiled library)
- lib/libmwalib.so     (Dynamic library for Linux)
- lib/libmwalib.dylib  (Dynamic library for MacOS)
- include/mwalib.h     (C Header file)
- CHANGELOG.md         (Change log for this and previous relases)
- LICENSE              (License for using mwalib in your projects)
- LICENSE-cfitsio      (Since libcfitsio is statically compiled into our static and dynamic libraries, we also include it's license)

NOTE: from release 0.3.2 onwards, libcfitsio is statically linked to mwalib in order to reduce issues with conflicting/incompatible versions of cfitsio. Therefore, there is no need for you to have cfitsio installed on your machine unless you are compiling from source.

To install on most Linux x86/64 distributions, the following would be all that is needed:

- Download release from mwalib [github releases](https://github.com/MWATelescope/mwalib/releases). (Where X.Y.Z is the current release version)

   ```bash
   wget "https://github.com/MWATelescope/mwalib/releases/download/vX.Y.Z/libmwalib-X.Y.Z-linux_x86_64.tar.gz" -O mwalib.tar.gz
   ```

- Untar the tarball

   ```bash
   mkdir mwalib
   tar xvf mwalib.tar.gz -C mwalib
   ```

- Install
  
  ```bash
  sudo cp mwalib/lib/libmwalib.* /usr/local/lib
  sudo cp mwalib/include/libmwalib.h /usr/local/include
  ```

- Register the library with ldconfig

 ```bash
 sudo ldconfig
 ```

## Example test output

```CorrelatorContext (
    Correlator version:       v1 Legacy,

    MWA latitude:             -26.703319405555554 degrees,
    MWA longitude:            116.67081523611111 degrees
    MWA altitude:             377.827 m,

    obsid:                    1065880128,

    Creator:                  DJacobs,
    Project ID:               G0009,
    Observation Name:         high_season1_2456581,
    Receivers:                [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
    Delays:                   [0, 2, 4, 6, 0, 2, 4, 6, 0, 2, 4, 6, 0, 2, 4, 6],
    Global attenuation:       1 dB,

    Scheduled start (UNIX)    1381844912,
    Scheduled end (UNIX)      1381845024,
    Scheduled start (GPS)     1065880128,
    Scheduled end (GPS)       1065880240,
    Scheduled start (utc)     2013-10-15 13:48:32 +00:00,
    Scheduled end (utc)       2013-10-15 13:50:24 +00:00,
    Scheduled start (MJD)     56580.57537037037,
    Scheduled end (MJD)       56580.57666666666,
    Scheduled duration        112 s,
    Actual UNIX start time:   1381844910,
    Actual UNIX end time:     1381845019,
    Actual duration:          109 s,
    Quack time:               0.5 s,
    Good UNIX start time:     1381844912.5,

    R.A. (tile_pointing):     3.166573685252408 degrees,
    Dec. (tile_pointing):     -25.96194063106248 degrees,
    R.A. (phase center):      0 degrees,
    Dec. (phase center):      -27 degrees,
    Azimuth:                  90 degrees,
    Altitude:                 76.2838 degrees,
    Sun altitude:             -42.5092395827051 degrees,
    Sun distance:             141.518031617465 degrees,
    Moon distance:            32.1177391815099 degrees,
    Jupiter distance:         114.851619013396 degrees,
    LST:                      348.061732169994 degrees,
    Hour angle:               -22:58:52.56 degrees,
    Grid name:                EOR1,
    Grid number:              3,

    num timesteps:            218,
    timesteps:                [unix=1381844910.000, unix=1381844910.500, unix=1381844911.000, unix=1381844911.500, unix=1381844912.000, unix=1381844912.500, unix=1381844913.000, unix=1381844913.500, unix=1381844914.000, unix=1381844914.500, unix=1381844915.000, unix=1381844915.500, unix=1381844916.000, unix=1381844916.500, unix=1381844917.000, unix=1381844917.500, unix=1381844918.000, unix=1381844918.500, unix=1381844919.000, unix=1381844919.500, unix=1381844920.000, unix=1381844920.500, unix=1381844921.000, unix=1381844921.500, unix=1381844922.000, unix=1381844922.500, unix=1381844923.000, unix=1381844923.500, unix=1381844924.000, unix=1381844924.500, unix=1381844925.000, unix=1381844925.500, unix=1381844926.000, unix=1381844926.500, unix=1381844927.000, unix=1381844927.500, unix=1381844928.000, unix=1381844928.500, unix=1381844929.000, unix=1381844929.500, unix=1381844930.000, unix=1381844930.500, unix=1381844931.000, unix=1381844931.500, unix=1381844932.000, unix=1381844932.500, unix=1381844933.000, unix=1381844933.500, unix=1381844934.000, unix=1381844934.500, unix=1381844935.000, unix=1381844935.500, unix=1381844936.000, unix=1381844936.500, unix=1381844937.000, unix=1381844937.500, unix=1381844938.000, unix=1381844938.500, unix=1381844939.000, unix=1381844939.500, unix=1381844940.000, unix=1381844940.500, unix=1381844941.000, unix=1381844941.500, unix=1381844942.000, unix=1381844942.500, unix=1381844943.000, unix=1381844943.500, unix=1381844944.000, unix=1381844944.500, unix=1381844945.000, unix=1381844945.500, unix=1381844946.000, unix=1381844946.500, unix=1381844947.000, unix=1381844947.500, unix=1381844948.000, unix=1381844948.500, unix=1381844949.000, unix=1381844949.500, unix=1381844950.000, unix=1381844950.500, unix=1381844951.000, unix=1381844951.500, unix=1381844952.000, unix=1381844952.500, unix=1381844953.000, unix=1381844953.500, unix=1381844954.000, unix=1381844954.500, unix=1381844955.000, unix=1381844955.500, unix=1381844956.000, unix=1381844956.500, unix=1381844957.000, unix=1381844957.500, unix=1381844958.000, unix=1381844958.500, unix=1381844959.000, unix=1381844959.500, unix=1381844960.000, unix=1381844960.500, unix=1381844961.000, unix=1381844961.500, unix=1381844962.000, unix=1381844962.500, unix=1381844963.000, unix=1381844963.500, unix=1381844964.000, unix=1381844964.500, unix=1381844965.000, unix=1381844965.500, unix=1381844966.000, unix=1381844966.500, unix=1381844967.000, unix=1381844967.500, unix=1381844968.000, unix=1381844968.500, unix=1381844969.000, unix=1381844969.500, unix=1381844970.000, unix=1381844970.500, unix=1381844971.000, unix=1381844971.500, unix=1381844972.000, unix=1381844972.500, unix=1381844973.000, unix=1381844973.500, unix=1381844974.000, unix=1381844974.500, unix=1381844975.000, unix=1381844975.500, unix=1381844976.000, unix=1381844976.500, unix=1381844977.000, unix=1381844977.500, unix=1381844978.000, unix=1381844978.500, unix=1381844979.000, unix=1381844979.500, unix=1381844980.000, unix=1381844980.500, unix=1381844981.000, unix=1381844981.500, unix=1381844982.000, unix=1381844982.500, unix=1381844983.000, unix=1381844983.500, unix=1381844984.000, unix=1381844984.500, unix=1381844985.000, unix=1381844985.500, unix=1381844986.000, unix=1381844986.500, unix=1381844987.000, unix=1381844987.500, unix=1381844988.000, unix=1381844988.500, unix=1381844989.000, unix=1381844989.500, unix=1381844990.000, unix=1381844990.500, unix=1381844991.000, unix=1381844991.500, unix=1381844992.000, unix=1381844992.500, unix=1381844993.000, unix=1381844993.500, unix=1381844994.000, unix=1381844994.500, unix=1381844995.000, unix=1381844995.500, unix=1381844996.000, unix=1381844996.500, unix=1381844997.000, unix=1381844997.500, unix=1381844998.000, unix=1381844998.500, unix=1381844999.000, unix=1381844999.500, unix=1381845000.000, unix=1381845000.500, unix=1381845001.000, unix=1381845001.500, unix=1381845002.000, unix=1381845002.500, unix=1381845003.000, unix=1381845003.500, unix=1381845004.000, unix=1381845004.500, unix=1381845005.000, unix=1381845005.500, unix=1381845006.000, unix=1381845006.500, unix=1381845007.000, unix=1381845007.500, unix=1381845008.000, unix=1381845008.500, unix=1381845009.000, unix=1381845009.500, unix=1381845010.000, unix=1381845010.500, unix=1381845011.000, unix=1381845011.500, unix=1381845012.000, unix=1381845012.500, unix=1381845013.000, unix=1381845013.500, unix=1381845014.000, unix=1381845014.500, unix=1381845015.000, unix=1381845015.500, unix=1381845016.000, unix=1381845016.500, unix=1381845017.000, unix=1381845017.500, unix=1381845018.000, unix=1381845018.500],

    num antennas:             128,
    antennas:                 [Tile011, Tile012, Tile013, Tile014, Tile015, Tile016, Tile017, Tile018, Tile021, Tile022, Tile023, Tile024, Tile025, Tile026, Tile027, Tile028, Tile031, Tile032, Tile033, Tile034, Tile035, Tile036, Tile037, Tile038, Tile041, Tile042, Tile043, Tile044, Tile045, Tile046, Tile047, Tile048, Tile051, Tile052, Tile053, Tile054, Tile055, Tile056, Tile057, Tile058, Tile061, Tile062, Tile063, Tile064, Tile065, Tile066, Tile067, Tile068, Tile071, Tile072, Tile073, Tile074, Tile075, Tile076, Tile077, Tile078, Tile081, Tile082, Tile083, Tile084, Tile085, Tile086, Tile087, Tile088, Tile091, Tile092, Tile093, Tile094, Tile095, Tile096, Tile097, Tile098, Tile101, Tile102, Tile103, Tile104, Tile105, Tile106, Tile107, Tile108, Tile111, Tile112, Tile113, Tile114, Tile115, Tile116, Tile117, Tile118, Tile121, Tile122, Tile123, Tile124, Tile125, Tile126, Tile127, Tile128, Tile131, Tile132, Tile133, Tile134, Tile135, Tile136, Tile137, Tile138, Tile141, Tile142, Tile143, Tile144, Tile145, Tile146, Tile147, Tile148, Tile151, Tile152, Tile153, Tile154, Tile155, Tile156, Tile157, Tile158, Tile161, Tile162, Tile163, Tile164, Tile165, Tile166, Tile167, Tile168],
    rf_inputs:                [Tile011X, Tile011Y, Tile012X, Tile012Y, Tile013X, Tile013Y, Tile014X, Tile014Y, Tile015X, Tile015Y, Tile016X, Tile016Y, Tile017X, Tile017Y, Tile018X, Tile018Y, Tile021X, Tile021Y, Tile022X, Tile022Y, Tile023X, Tile023Y, Tile024X, Tile024Y, Tile025X, Tile025Y, Tile026X, Tile026Y, Tile027X, Tile027Y, Tile028X, Tile028Y, Tile031X, Tile031Y, Tile032X, Tile032Y, Tile033X, Tile033Y, Tile034X, Tile034Y, Tile035X, Tile035Y, Tile036X, Tile036Y, Tile037X, Tile037Y, Tile038X, Tile038Y, Tile041X, Tile041Y, Tile042X, Tile042Y, Tile043X, Tile043Y, Tile044X, Tile044Y, Tile045X, Tile045Y, Tile046X, Tile046Y, Tile047X, Tile047Y, Tile048X, Tile048Y, Tile051X, Tile051Y, Tile052X, Tile052Y, Tile053X, Tile053Y, Tile054X, Tile054Y, Tile055X, Tile055Y, Tile056X, Tile056Y, Tile057X, Tile057Y, Tile058X, Tile058Y, Tile061X, Tile061Y, Tile062X, Tile062Y, Tile063X, Tile063Y, Tile064X, Tile064Y, Tile065X, Tile065Y, Tile066X, Tile066Y, Tile067X, Tile067Y, Tile068X, Tile068Y, Tile071X, Tile071Y, Tile072X, Tile072Y, Tile073X, Tile073Y, Tile074X, Tile074Y, Tile075X, Tile075Y, Tile076X, Tile076Y, Tile077X, Tile077Y, Tile078X, Tile078Y, Tile081X, Tile081Y, Tile082X, Tile082Y, Tile083X, Tile083Y, Tile084X, Tile084Y, Tile085X, Tile085Y, Tile086X, Tile086Y, Tile087X, Tile087Y, Tile088X, Tile088Y, Tile091X, Tile091Y, Tile092X, Tile092Y, Tile093X, Tile093Y, Tile094X, Tile094Y, Tile095X, Tile095Y, Tile096X, Tile096Y, Tile097X, Tile097Y, Tile098X, Tile098Y, Tile101X, Tile101Y, Tile102X, Tile102Y, Tile103X, Tile103Y, Tile104X, Tile104Y, Tile105X, Tile105Y, Tile106X, Tile106Y, Tile107X, Tile107Y, Tile108X, Tile108Y, Tile111X, Tile111Y, Tile112X, Tile112Y, Tile113X, Tile113Y, Tile114X, Tile114Y, Tile115X, Tile115Y, Tile116X, Tile116Y, Tile117X, Tile117Y, Tile118X, Tile118Y, Tile121X, Tile121Y, Tile122X, Tile122Y, Tile123X, Tile123Y, Tile124X, Tile124Y, Tile125X, Tile125Y, Tile126X, Tile126Y, Tile127X, Tile127Y, Tile128X, Tile128Y, Tile131X, Tile131Y, Tile132X, Tile132Y, Tile133X, Tile133Y, Tile134X, Tile134Y, Tile135X, Tile135Y, Tile136X, Tile136Y, Tile137X, Tile137Y, Tile138X, Tile138Y, Tile141X, Tile141Y, Tile142X, Tile142Y, Tile143X, Tile143Y, Tile144X, Tile144Y, Tile145X, Tile145Y, Tile146X, Tile146Y, Tile147X, Tile147Y, Tile148X, Tile148Y, Tile151X, Tile151Y, Tile152X, Tile152Y, Tile153X, Tile153Y, Tile154X, Tile154Y, Tile155X, Tile155Y, Tile156X, Tile156Y, Tile157X, Tile157Y, Tile158X, Tile158Y, Tile161X, Tile161Y, Tile162X, Tile162Y, Tile163X, Tile163Y, Tile164X, Tile164Y, Tile165X, Tile165Y, Tile166X, Tile166Y, Tile167X, Tile167Y, Tile168X, Tile168Y],

    num baselines:            8256,
    baselines:                0 v 0 to 127 v 127
    num auto-correlations:    128,
    num cross-correlations:   8128,

    num antenna pols:         2,
    num visibility pols:      4,
    visibility pols:          XX, XY, YX, YY,

    observation bandwidth:    30.72 MHz,
    num coarse channels,      24,
    coarse channels:          [gpu=24 corr=23 rec=131 @ 167.680 MHz, gpu=23 corr=22 rec=132 @ 168.960 MHz, gpu=22 corr=21 rec=133 @ 170.240 MHz, gpu=21 corr=20 rec=134 @ 171.520 MHz, gpu=20 corr=19 rec=135 @ 172.800 MHz, gpu=19 corr=18 rec=136 @ 174.080 MHz, gpu=18 corr=17 rec=137 @ 175.360 MHz, gpu=17 corr=16 rec=138 @ 176.640 MHz, gpu=16 corr=15 rec=139 @ 177.920 MHz, gpu=15 corr=14 rec=140 @ 179.200 MHz, gpu=14 corr=13 rec=141 @ 180.480 MHz, gpu=13 corr=12 rec=142 @ 181.760 MHz, gpu=12 corr=11 rec=143 @ 183.040 MHz, gpu=11 corr=10 rec=144 @ 184.320 MHz, gpu=10 corr=9 rec=145 @ 185.600 MHz, gpu=9 corr=8 rec=146 @ 186.880 MHz, gpu=8 corr=7 rec=147 @ 188.160 MHz, gpu=7 corr=6 rec=148 @ 189.440 MHz, gpu=6 corr=5 rec=149 @ 190.720 MHz, gpu=5 corr=4 rec=150 @ 192.000 MHz, gpu=4 corr=3 rec=151 @ 193.280 MHz, gpu=3 corr=2 rec=152 @ 194.560 MHz, gpu=2 corr=1 rec=153 @ 195.840 MHz, gpu=1 corr=0 rec=154 @ 197.120 MHz],

    Correlator Mode:
    Mode:                     HW_LFILES,
    fine channel resolution:  40 kHz,
    integration time:         0.50 s
    num fine channels/coarse: 32,

    gpubox HDU size:          8.0625 MiB,
    Memory usage per scan:    387 MiB,

    metafits filename:        1065880128.metafits,
    gpubox batches:           [
    batch_number=0 gpubox_files=[filename=1065880128_20131015134830_gpubox01_00.fits channelidentifier=1, filename=1065880128_20131015134830_gpubox02_00.fits channelidentifier=2, filename=1065880128_20131015134830_gpubox03_00.fits channelidentifier=3, filename=1065880128_20131015134830_gpubox04_00.fits channelidentifier=4, filename=1065880128_20131015134830_gpubox05_00.fits channelidentifier=5, filename=1065880128_20131015134830_gpubox06_00.fits channelidentifier=6, filename=1065880128_20131015134830_gpubox07_00.fits channelidentifier=7, filename=1065880128_20131015134830_gpubox08_00.fits channelidentifier=8, filename=1065880128_20131015134830_gpubox09_00.fits channelidentifier=9, filename=1065880128_20131015134830_gpubox10_00.fits channelidentifier=10, filename=1065880128_20131015134830_gpubox11_00.fits channelidentifier=11, filename=1065880128_20131015134830_gpubox12_00.fits channelidentifier=12, filename=1065880128_20131015134830_gpubox13_00.fits channelidentifier=13, filename=1065880128_20131015134830_gpubox14_00.fits channelidentifier=14, filename=1065880128_20131015134830_gpubox15_00.fits channelidentifier=15, filename=1065880128_20131015134830_gpubox16_00.fits channelidentifier=16, filename=1065880128_20131015134830_gpubox17_00.fits channelidentifier=17, filename=1065880128_20131015134830_gpubox18_00.fits channelidentifier=18, filename=1065880128_20131015134830_gpubox19_00.fits channelidentifier=19, filename=1065880128_20131015134830_gpubox20_00.fits channelidentifier=20, filename=1065880128_20131015134830_gpubox21_00.fits channelidentifier=21, filename=1065880128_20131015134830_gpubox22_00.fits channelidentifier=22, filename=1065880128_20131015134830_gpubox23_00.fits channelidentifier=23, filename=1065880128_20131015134830_gpubox24_00.fits channelidentifier=24],
    batch_number=1 gpubox_files=[filename=1065880128_20131015134930_gpubox01_01.fits channelidentifier=1, filename=1065880128_20131015134930_gpubox02_01.fits channelidentifier=2, filename=1065880128_20131015134930_gpubox03_01.fits channelidentifier=3, filename=1065880128_20131015134930_gpubox04_01.fits channelidentifier=4, filename=1065880128_20131015134930_gpubox05_01.fits channelidentifier=5, filename=1065880128_20131015134930_gpubox06_01.fits channelidentifier=6, filename=1065880128_20131015134930_gpubox07_01.fits channelidentifier=7, filename=1065880128_20131015134930_gpubox08_01.fits channelidentifier=8, filename=1065880128_20131015134930_gpubox09_01.fits channelidentifier=9, filename=1065880128_20131015134930_gpubox10_01.fits channelidentifier=10, filename=1065880128_20131015134930_gpubox11_01.fits channelidentifier=11, filename=1065880128_20131015134930_gpubox12_01.fits channelidentifier=12, filename=1065880128_20131015134930_gpubox13_01.fits channelidentifier=13, filename=1065880128_20131015134930_gpubox14_01.fits channelidentifier=14, filename=1065880128_20131015134930_gpubox15_01.fits channelidentifier=15, filename=1065880128_20131015134930_gpubox16_01.fits channelidentifier=16, filename=1065880128_20131015134930_gpubox17_01.fits channelidentifier=17, filename=1065880128_20131015134930_gpubox18_01.fits channelidentifier=18, filename=1065880128_20131015134930_gpubox19_01.fits channelidentifier=19, filename=1065880128_20131015134930_gpubox20_01.fits channelidentifier=20, filename=1065880128_20131015134930_gpubox21_01.fits channelidentifier=21, filename=1065880128_20131015134930_gpubox22_01.fits channelidentifier=22, filename=1065880128_20131015134930_gpubox23_01.fits channelidentifier=23, filename=1065880128_20131015134930_gpubox24_01.fits channelidentifier=24],
],
)
```
