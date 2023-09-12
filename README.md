# mwalib

<div class="bg-gray-dark" align="center" style="background-color:#24292e">
<img src="img/mwalib_logo.png" alt="mwalib logo" height="200px"/>
</div>

![Tests](https://github.com/MWATelescope/mwalib/workflows/Cross-platform%20tests/badge.svg)
![Python tests](https://github.com/MWATelescope/mwalib/workflows/Python%20tests/badge.svg)
![Code Coverage](https://github.com/MWATelescope/mwalib/workflows/Code%20Coverage/badge.svg)
[![codecov](https://codecov.io/gh/MWATelescope/mwalib/branch/main/graph/badge.svg)](https://app.codecov.io/gh/MWATelescope/mwalib/)
[![Crates.io](https://img.shields.io/crates/v/mwalib)](https://crates.io/crates/mwalib)
![Crates.io](https://img.shields.io/crates/d/mwalib)
![Crates.io](https://img.shields.io/crates/l/mwalib)
[![docs](https://docs.rs/mwalib/badge.svg)](https://docs.rs/crate/mwalib/latest)
[![Rust Report Card](https://rust-reportcard.xuri.me/badge/github.com/MWATelescope/mwalib)](https://rust-reportcard.xuri.me/report/github.com/MWATelescope/mwalib)

mwalib is an MWA library to read raw visibilities, voltages and metadata into a common structure.
mwalib supports the existing "legacy" MWA correlator, as well as the "MWAX" correlator observations. This library
strives to provide a single interface to work with all incarnations of MWA metadata, correlator and
voltage formats and abstract away the nitty gritty details about reading MWA data. The only exception
is that raw legacy VCS data which has not been recombined is not currently supported.

mwalib is a library you can use in:

* Rust (see examples/*.rs)
* C (see examples/*.c)
* Python (see examples/*.py)

mwalib provides the following binaries and implementations with releases starting from v1.0.0 onwards:

| Filename  | Purpose  |
|-----------|----------|
|mwalib-vX.X.X-linux-x86-64.tar.gz | Linux "C"/"C++" compatible library - includes ".h" and "libmwalib.a" and "libmwalib.so" library files. |
|mwalib-vX.X.X-linux-python-x86-64.tar.gz | Linux compatible Python wheel. |
|mwalib-vX.X.X-macosx-x86-64.tar.gz | MacOSX "C"/"C++" compatible library - includes ".h" and "libmwalib.a" and "libmwalib.so" library files. |
|mwalib-vX.X.X-macosx-python-x86-64.tar.gz | MacOSX compatible Python wheel. |

NOTE: for both Linux, more optimised builds are named with a v2 and v3 after the "x86-64". These suffixes denote that the compilation was optimised for the specified [CPU microarchitecture levels](https://en.wikipedia.org/wiki/X86-64#Microarchitecture_levels), with "x86-64" being the most compatible with the very oldest CPUs. If you have a CPU which was manufactured after 2008 you are safe with x86-64-v2, and if it is newer than 2015 then you are safe using x86-64-v3.

----
For installation instructions, concepts and usage info, please see the [`mwalib GitHub Wiki`](https://github.com/MWATelescope/mwalib/wiki).

----

Be sure to also check out these related repositories:

* [`Birli`](https://github.com/MWATelescope/Birli) - A Murchison Widefield Array (MWA) pre-processing pipeline.
* [`Marlu`](https://github.com/MWATelescope/Marlu) - Convenience Rust code that handles coordinate transformations, Jones matrices, etc.
* [`Hyperdrive`](https://github.com/MWATelescope/mwa_hyperdrive) - Calibration software for the Murchison Widefield Array (MWA) radio telescope.

## Example test output

```text
CorrelatorContext (
            Metafits Context:           MetafitsContext (
    obsid:                    1101503312,
    mode:                     HW_LFILES,

    Correlator Mode:
    fine channel resolution:  10 kHz,
    integration time:         2.00 s
    num fine channels/coarse: 128,

    Geometric delays applied          : No,
    Cable length corrections applied  : false,
    Calibration delays & gains applied: false,

    Creator:                  Randall,
    Project ID:               G0009,
    Observation Name:         FDS_DEC-26.7_121,
    Receivers:                [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
    Delays:                   [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    Global attenuation:       1 dB,

    Scheduled start (UNIX)    1417468096,
    Scheduled end (UNIX)      1417468208,
    Scheduled start (GPS)     1101503312,
    Scheduled end (GPS)       1101503424,
    Scheduled start (utc)     2014-12-01 21:08:16 +00:00,
    Scheduled end (utc)       2014-12-01 21:10:08 +00:00,
    Scheduled start (MJD)     56992.88074074074,
    Scheduled end (MJD)       56992.88203703703,
    Scheduled duration        112 s,
    Quack time:               2 s,
    Good UNIX start time:     1417468098,

    Timesteps:                [unix=1417468096.000, gps=1101503312.000, unix=1417468098.000, gps=1101503314.000, unix=1417468100.000, gps=1101503316.000, unix=1417468102.000, gps=1101503318.000, unix=1417468104.000, gps=1101503320.000, unix=1417468106.000, gps=1101503322.000, unix=1417468108.000, gps=1101503324.000, unix=1417468110.000, gps=1101503326.000, unix=1417468112.000, gps=1101503328.000, unix=1417468114.000, gps=1101503330.000, unix=1417468116.000, gps=1101503332.000, unix=1417468118.000, gps=1101503334.000, unix=1417468120.000, gps=1101503336.000, unix=1417468122.000, gps=1101503338.000, unix=1417468124.000, gps=1101503340.000, unix=1417468126.000, gps=1101503342.000, unix=1417468128.000, gps=1101503344.000, unix=1417468130.000, gps=1101503346.000, unix=1417468132.000, gps=1101503348.000, unix=1417468134.000, gps=1101503350.000, unix=1417468136.000, gps=1101503352.000, unix=1417468138.000, gps=1101503354.000, unix=1417468140.000, gps=1101503356.000, unix=1417468142.000, gps=1101503358.000, unix=1417468144.000, gps=1101503360.000, unix=1417468146.000, gps=1101503362.000, unix=1417468148.000, gps=1101503364.000, unix=1417468150.000, gps=1101503366.000, unix=1417468152.000, gps=1101503368.000, unix=1417468154.000, gps=1101503370.000, unix=1417468156.000, gps=1101503372.000, unix=1417468158.000, gps=1101503374.000, unix=1417468160.000, gps=1101503376.000, unix=1417468162.000, gps=1101503378.000, unix=1417468164.000, gps=1101503380.000, unix=1417468166.000, gps=1101503382.000, unix=1417468168.000, gps=1101503384.000, unix=1417468170.000, gps=1101503386.000, unix=1417468172.000, gps=1101503388.000, unix=1417468174.000, gps=1101503390.000, unix=1417468176.000, gps=1101503392.000, unix=1417468178.000, gps=1101503394.000, unix=1417468180.000, gps=1101503396.000, unix=1417468182.000, gps=1101503398.000, unix=1417468184.000, gps=1101503400.000, unix=1417468186.000, gps=1101503402.000, unix=1417468188.000, gps=1101503404.000, unix=1417468190.000, gps=1101503406.000, unix=1417468192.000, gps=1101503408.000, unix=1417468194.000, gps=1101503410.000, unix=1417468196.000, gps=1101503412.000, unix=1417468198.000, gps=1101503414.000, unix=1417468200.000, gps=1101503416.000, unix=1417468202.000, gps=1101503418.000, unix=1417468204.000, gps=1101503420.000, unix=1417468206.000, gps=1101503422.000],

    Coarse Channels:          [gpu=1 corr=0 rec=109 @ 139.520 MHz, gpu=2 corr=1 rec=110 @ 140.800 MHz, gpu=3 corr=2 rec=111 @ 142.080 MHz, gpu=4 corr=3 rec=112 @ 143.360 MHz, gpu=5 corr=4 rec=113 @ 144.640 MHz, gpu=6 corr=5 rec=114 @ 145.920 MHz, gpu=7 corr=6 rec=115 @ 147.200 MHz, gpu=8 corr=7 rec=116 @ 148.480 MHz, gpu=9 corr=8 rec=117 @ 149.760 MHz, gpu=10 corr=9 rec=118 @ 151.040 MHz, gpu=11 corr=10 rec=119 @ 152.320 MHz, gpu=12 corr=11 rec=120 @ 153.600 MHz, gpu=13 corr=12 rec=121 @ 154.880 MHz, gpu=14 corr=13 rec=122 @ 156.160 MHz, gpu=15 corr=14 rec=123 @ 157.440 MHz, gpu=16 corr=15 rec=124 @ 158.720 MHz, gpu=17 corr=16 rec=125 @ 160.000 MHz, gpu=18 corr=17 rec=126 @ 161.280 MHz, gpu=19 corr=18 rec=127 @ 162.560 MHz, gpu=20 corr=19 rec=128 @ 163.840 MHz, gpu=24 corr=23 rec=129 @ 165.120 MHz, gpu=23 corr=22 rec=130 @ 166.400 MHz, gpu=22 corr=21 rec=131 @ 167.680 MHz, gpu=21 corr=20 rec=132 @ 168.960 MHz],

    R.A. (tile_pointing):     144.2107504850443 degrees,
    Dec. (tile_pointing):     -26.63403125476213 degrees,
    R.A. (phase center):      Some(None) degrees,
    Dec. (phase center):      Some(None) degrees,
    Azimuth:                  0 degrees,
    Altitude:                 90 degrees,
    Sun altitude:             -1.53222775573148 degrees,
    Sun distance:             91.5322277557315 degrees,
    Moon distance:            131.880015235607 degrees,
    Jupiter distance:         41.401684338269 degrees,
    LST:                      144.381251875516 degrees,
    Hour angle:               -00:00:00.00 degrees,
    Grid name:                sweet,
    Grid number:              0,

    num antennas:             128,
    antennas:                 [Tile011, Tile012, Tile013, Tile014, Tile015, Tile016, Tile017, Tile018, Tile021, Tile022, Tile023, Tile024, Tile025, Tile026, Tile027, Tile028, Tile031, Tile032, Tile033, Tile034, Tile035, Tile036, Tile037, Tile038, Tile041, Tile042, Tile043, Tile044, Tile045, Tile046, Tile047, Tile048, Tile051, Tile052, Tile053, Tile054, Tile055, Tile056, Tile057, Tile058, Tile061, Tile062, Tile063, Tile064, Tile065, Tile066, Tile067, Tile068, Tile071, Tile072, Tile073, Tile074, Tile075, Tile076, Tile077, Tile078, Tile081, Tile082, Tile083, Tile084, Tile085, Tile086, Tile087, Tile088, Tile091, Tile092, Tile093, Tile094, Tile095, Tile096, Tile097, Tile098, Tile101, Tile102, Tile103, Tile104, Tile105, Tile106, Tile107, Tile108, Tile111, Tile112, Tile113, Tile114, Tile115, Tile116, Tile117, Tile118, Tile121, Tile122, Tile123, Tile124, Tile125, Tile126, Tile127, Tile128, Tile131, Tile132, Tile133, Tile134, Tile135, Tile136, Tile137, Tile138, Tile141, Tile142, Tile143, Tile144, Tile145, Tile146, Tile147, Tile148, Tile151, Tile152, Tile153, Tile154, Tile155, Tile156, Tile157, Tile158, Tile161, Tile162, Tile163, Tile164, Tile165, Tile166, Tile167, Tile168],
    rf_inputs:                [Tile011X, Tile011Y, Tile012X, Tile012Y, Tile013X, Tile013Y, Tile014X, Tile014Y, Tile015X, Tile015Y, Tile016X, Tile016Y, Tile017X, Tile017Y, Tile018X, Tile018Y, Tile021X, Tile021Y, Tile022X, Tile022Y, Tile023X, Tile023Y, Tile024X, Tile024Y, Tile025X, Tile025Y, Tile026X, Tile026Y, Tile027X, Tile027Y, Tile028X, Tile028Y, Tile031X, Tile031Y, Tile032X, Tile032Y, Tile033X, Tile033Y, Tile034X, Tile034Y, Tile035X, Tile035Y, Tile036X, Tile036Y, Tile037X, Tile037Y, Tile038X, Tile038Y, Tile041X, Tile041Y, Tile042X, Tile042Y, Tile043X, Tile043Y, Tile044X, Tile044Y, Tile045X, Tile045Y, Tile046X, Tile046Y, Tile047X, Tile047Y, Tile048X, Tile048Y, Tile051X, Tile051Y, Tile052X, Tile052Y, Tile053X, Tile053Y, Tile054X, Tile054Y, Tile055X, Tile055Y, Tile056X, Tile056Y, Tile057X, Tile057Y, Tile058X, Tile058Y, Tile061X, Tile061Y, Tile062X, Tile062Y, Tile063X, Tile063Y, Tile064X, Tile064Y, Tile065X, Tile065Y, Tile066X, Tile066Y, Tile067X, Tile067Y, Tile068X, Tile068Y, Tile071X, Tile071Y, Tile072X, Tile072Y, Tile073X, Tile073Y, Tile074X, Tile074Y, Tile075X, Tile075Y, Tile076X, Tile076Y, Tile077X, Tile077Y, Tile078X, Tile078Y, Tile081X, Tile081Y, Tile082X, Tile082Y, Tile083X, Tile083Y, Tile084X, Tile084Y, Tile085X, Tile085Y, Tile086X, Tile086Y, Tile087X, Tile087Y, Tile088X, Tile088Y, Tile091X, Tile091Y, Tile092X, Tile092Y, Tile093X, Tile093Y, Tile094X, Tile094Y, Tile095X, Tile095Y, Tile096X, Tile096Y, Tile097X, Tile097Y, Tile098X, Tile098Y, Tile101X, Tile101Y, Tile102X, Tile102Y, Tile103X, Tile103Y, Tile104X, Tile104Y, Tile105X, Tile105Y, Tile106X, Tile106Y, Tile107X, Tile107Y, Tile108X, Tile108Y, Tile111X, Tile111Y, Tile112X, Tile112Y, Tile113X, Tile113Y, Tile114X, Tile114Y, Tile115X, Tile115Y, Tile116X, Tile116Y, Tile117X, Tile117Y, Tile118X, Tile118Y, Tile121X, Tile121Y, Tile122X, Tile122Y, Tile123X, Tile123Y, Tile124X, Tile124Y, Tile125X, Tile125Y, Tile126X, Tile126Y, Tile127X, Tile127Y, Tile128X, Tile128Y, Tile131X, Tile131Y, Tile132X, Tile132Y, Tile133X, Tile133Y, Tile134X, Tile134Y, Tile135X, Tile135Y, Tile136X, Tile136Y, Tile137X, Tile137Y, Tile138X, Tile138Y, Tile141X, Tile141Y, Tile142X, Tile142Y, Tile143X, Tile143Y, Tile144X, Tile144Y, Tile145X, Tile145Y, Tile146X, Tile146Y, Tile147X, Tile147Y, Tile148X, Tile148Y, Tile151X, Tile151Y, Tile152X, Tile152Y, Tile153X, Tile153Y, Tile154X, Tile154Y, Tile155X, Tile155Y, Tile156X, Tile156Y, Tile157X, Tile157Y, Tile158X, Tile158Y, Tile161X, Tile161Y, Tile162X, Tile162Y, Tile163X, Tile163Y, Tile164X, Tile164Y, Tile165X, Tile165Y, Tile166X, Tile166Y, Tile167X, Tile167Y, Tile168X, Tile168Y],

    num antenna pols:         2,
    num baselines:            8256,
    baselines:                0 v 0 to 127 v 127
    num auto-correlations:    128,
    num cross-correlations:   8128,

    num visibility pols:      4,
    visibility pols:          XX, XY, YX, YY,

    metafits FREQCENT key:    154.24 MHz,

    metafits filename:        test_files/1101503312_1_timestep/1101503312.metafits,
)

            MWA version:                Correlator v1 Legacy,

            num timesteps:              56,
            timesteps:                  [unix=1417468096.000, gps=1101503312.000, unix=1417468098.000, gps=1101503314.000, unix=1417468100.000, gps=1101503316.000, unix=1417468102.000, gps=1101503318.000, unix=1417468104.000, gps=1101503320.000, unix=1417468106.000, gps=1101503322.000, unix=1417468108.000, gps=1101503324.000, unix=1417468110.000, gps=1101503326.000, unix=1417468112.000, gps=1101503328.000, unix=1417468114.000, gps=1101503330.000, unix=1417468116.000, gps=1101503332.000, unix=1417468118.000, gps=1101503334.000, unix=1417468120.000, gps=1101503336.000, unix=1417468122.000, gps=1101503338.000, unix=1417468124.000, gps=1101503340.000, unix=1417468126.000, gps=1101503342.000, unix=1417468128.000, gps=1101503344.000, unix=1417468130.000, gps=1101503346.000, unix=1417468132.000, gps=1101503348.000, unix=1417468134.000, gps=1101503350.000, unix=1417468136.000, gps=1101503352.000, unix=1417468138.000, gps=1101503354.000, unix=1417468140.000, gps=1101503356.000, unix=1417468142.000, gps=1101503358.000, unix=1417468144.000, gps=1101503360.000, unix=1417468146.000, gps=1101503362.000, unix=1417468148.000, gps=1101503364.000, unix=1417468150.000, gps=1101503366.000, unix=1417468152.000, gps=1101503368.000, unix=1417468154.000, gps=1101503370.000, unix=1417468156.000, gps=1101503372.000, unix=1417468158.000, gps=1101503374.000, unix=1417468160.000, gps=1101503376.000, unix=1417468162.000, gps=1101503378.000, unix=1417468164.000, gps=1101503380.000, unix=1417468166.000, gps=1101503382.000, unix=1417468168.000, gps=1101503384.000, unix=1417468170.000, gps=1101503386.000, unix=1417468172.000, gps=1101503388.000, unix=1417468174.000, gps=1101503390.000, unix=1417468176.000, gps=1101503392.000, unix=1417468178.000, gps=1101503394.000, unix=1417468180.000, gps=1101503396.000, unix=1417468182.000, gps=1101503398.000, unix=1417468184.000, gps=1101503400.000, unix=1417468186.000, gps=1101503402.000, unix=1417468188.000, gps=1101503404.000, unix=1417468190.000, gps=1101503406.000, unix=1417468192.000, gps=1101503408.000, unix=1417468194.000, gps=1101503410.000, unix=1417468196.000, gps=1101503412.000, unix=1417468198.000, gps=1101503414.000, unix=1417468200.000, gps=1101503416.000, unix=1417468202.000, gps=1101503418.000, unix=1417468204.000, gps=1101503420.000, unix=1417468206.000, gps=1101503422.000],
            num coarse channels,        24,
            coarse channels:            [gpu=1 corr=0 rec=109 @ 139.520 MHz, gpu=2 corr=1 rec=110 @ 140.800 MHz, gpu=3 corr=2 rec=111 @ 142.080 MHz, gpu=4 corr=3 rec=112 @ 143.360 MHz, gpu=5 corr=4 rec=113 @ 144.640 MHz, gpu=6 corr=5 rec=114 @ 145.920 MHz, gpu=7 corr=6 rec=115 @ 147.200 MHz, gpu=8 corr=7 rec=116 @ 148.480 MHz, gpu=9 corr=8 rec=117 @ 149.760 MHz, gpu=10 corr=9 rec=118 @ 151.040 MHz, gpu=11 corr=10 rec=119 @ 152.320 MHz, gpu=12 corr=11 rec=120 @ 153.600 MHz, gpu=13 corr=12 rec=121 @ 154.880 MHz, gpu=14 corr=13 rec=122 @ 156.160 MHz, gpu=15 corr=14 rec=123 @ 157.440 MHz, gpu=16 corr=15 rec=124 @ 158.720 MHz, gpu=17 corr=16 rec=125 @ 160.000 MHz, gpu=18 corr=17 rec=126 @ 161.280 MHz, gpu=19 corr=18 rec=127 @ 162.560 MHz, gpu=20 corr=19 rec=128 @ 163.840 MHz, gpu=24 corr=23 rec=129 @ 165.120 MHz, gpu=23 corr=22 rec=130 @ 166.400 MHz, gpu=22 corr=21 rec=131 @ 167.680 MHz, gpu=21 corr=20 rec=132 @ 168.960 MHz],

            provided timesteps indices:   1: [0],
            provided coarse chan indices: 1: [0],

            Common timestep indices:    1: [0],
            Common coarse chan indices: 1: [0],
            Common UNIX start time:     1417468096,
            Common UNIX end time:       1417468098,
            Common GPS start time:      1101503312,
            Common GPS end time:        1101503314,
            Common duration:            2 s,
            Common bandwidth:           1.28 MHz,

            Common/Good timestep indices:    0: [],
            Common/Good coarse chan indices: 0: [],
            Common/Good UNIX start time:     0,
            Common/Good UNIX end time:       0,
            Common/Good GPS start time:      0,
            Common/Good GPS end time:        0,
            Common/Good duration:            0 s,
            Common/Good bandwidth:           0 MHz,

            gpubox HDU size:            32.25 MiB,
            Memory usage per scan:      32.25 MiB,

            gpubox batches:             [
    batch_number=0 gpubox_files=[filename=test_files/1101503312_1_timestep/1101503312_20141201210818_gpubox01_00.fits channelidentifier=1],
],
        )
```

This repo is approved by...

<img src="img/CIRA_Rust_Evangelism_Strike_Force.png" height="200px" alt="CIRA Rust Evangelism Strike Force logo">
