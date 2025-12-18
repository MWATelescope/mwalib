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

mwalib is an MWA library to read raw visibilities, voltages and metadata into a common structure.
mwalib supports the existing "legacy" MWA correlator, as well as the "MWAX" correlator observations. This library
strives to provide a single interface to work with all incarnations of MWA metadata, correlator and
voltage formats and abstract away the nitty gritty details about reading MWA data. The only exception
is that raw legacy VCS data which has not been recombined (rearranged into per-coarse channel files) 
is not currently supported.

mwalib is a library you can use in:

* Rust (see examples/*.rs)
* C (see examples/*.c)
* Python (see examples/*.py)

## Installation and Usage

For installation instructions, concepts and usage info, please see the [`mwalib article on the MWATelescope Wiki`](https://mwatelescope.atlassian.net/wiki/spaces/MP/pages/348127236/mwalib).

## Related repositories

Be sure to also check out these related repositories which make use of mwalib:

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

    Timesteps:                [unix=1417468096.000, gps=1101503312.000, unix=1417468098.000, gps=1101503314.000...unix=1417468200.000, gps=1101503416.000, unix=1417468206.000, gps=1101503422.000],

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
    antennas:                 [Tile011, Tile012, Tile013, Tile014, Tile015...Tile166Y, Tile167X, Tile167Y, Tile168X, Tile168Y],

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
            timesteps:                  [unix=1417468096.000, gps=1101503312.000, unix=1417468098.000, gps=1101503314.000...unix=1417468204.000, gps=1101503420.000, unix=1417468206.000, gps=1101503422.000],
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
