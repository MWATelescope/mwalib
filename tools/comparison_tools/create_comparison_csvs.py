#!/usr/bin/env python
#
# create_comparison_csvs - dump basic visibility data to allow comparison between mwalib, pyuvdata and cotter
#
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.
#
# NOTE: this example requires numpy, pymwalib and pyuvdata packages. These can be installed via pip and the supplied
# requirements file.
#
# e.g. pip install -r requirements.txt
#
# Example Usage:
# $ python create_comparison_csvs.py -c /path/to/casa.ms -m /path/to/1234567890.metafits /path/to/1234567890_gpubox01_00.fits
#
# This will produce 3 csv files of the first timestep's visibilities, all in the same order:
# * Rows represent the baseline * fine channel.
#  * Baselines are 0v0, 0v1, 0v2 ... 0v127, 1v1, 1v2,...1v127, 2v2, 2v3... ...127v127.
#  * Fine channels are from 0...num_fine_chans.
#  * Thus the first row is baseline 0, fine_chan 0.
#  * Then.. baseline 0, fine_chan 1.
#  * ...
#  * baseline 0, fine_chan <last>.
#  * Then.. baseline 1, fine_chan 0... and so on.
#
# * Comma seperated values in each column of a row are xx_real, xx_imag, xy_r, xy_i, yx_r, yx_i, yy_r, yy_i.
#
# To create a useful CASA measurement set with cotter to enable visibility verification, run cotter with these args:
# $ cotter -nostats -noantennapruning -noflagautos -noflagdcchannels -norfi -nogeom -nosbgains -edgewidth 0 \
# -initflag 0 -sbpassband /path/to/sbpassbandfiles/10khz.txt -m /path/to/metafits.metafits \
# -o /path/to/output.ms /path/to/gpubox_files/*gpubox*.fits
#
# NOTES about cotter:
# * Cable delay corrections cannot be turned off via command line arg in cotter as of v4.5
#   We need cable delays off to compare raw visibilities. So I have a Docker container where I have commented out
#   the cable delay corrections, but other than that it is v4.5 of cotter. Get it here:
#   https://hub.docker.com/repository/docker/paladinsmeg/cotter45_no_cable_sleap_test
#
# * We also do not want cotter to correct the bandshape so we need to pass a special file of 1's as an argument.
#   For a 10kHz correlator mode, we need to pass a file containing 128 rows of 5 1's.
#   for a 20kHz correlator mode, we need to pass a file containing 64 rows of 5 1's.
#   For a 40kHz correlator mode, we need to pass a file containing 32 rows of 5 1's.
#
# Expected known (and accepted) differences in the data:
# * mwalib differs from pyuvdata: all imaginary values are conjugated (this is because of differing correlation triangles used).
# * mwalib differs from cotter: where ant1=ant2 cotter's values are conjugated with respect to mwalib.
# * mwalib and pyuvdata differ from cotter: Cotter sets XY to 0+0j for all cases where ant1==ant2.
#
import argparse
from pymwalib.correlator_context import CorrelatorContext
from pyuvdata import UVData
import casacore.tables

def get_baseline_from_antennas(antenna1, antenna2, num_antennas):
    baseline_index = 0
    for ant1 in range(0,num_antennas):
        for ant2 in range(ant1, num_antennas):
            if ant1 == antenna1 and ant2 == antenna2:
                return baseline_index

            baseline_index += 1

    # Baseline was not found at all
    return None

def dump_mwalib(ant1, ant2, timestep_index, fine_chan_index, fine_chan_count, gpuboxfiles, metafits, out_filename):
    print("pymwalib:")
    print("======================================")
    with CorrelatorContext(metafits, gpuboxfiles) as cc:
        # Get data
        data = cc.read_by_baseline(timestep_index, coarse_chan_index)

        if out_filename is None:
            baseline_index = get_baseline_from_antennas(ant1, ant2, 128)

            # print details
            print(
                f"Timestep[{timestep_index}]          = Unix time {cc.timesteps[timestep_index].unix_time_ms / 1000.0} GPS: {cc.timesteps[timestep_index].gps_time_ms / 1000.0}")
            print(
                f"Coarse Channel[{coarse_chan_index}]    = Reciever Chan {cc.coarse_channels[coarse_chan_index].rec_chan_number}, GPUBOX number {cc.coarse_channels[coarse_chan_index].gpubox_number}")
            print(f"Fine channels[{fine_chan_index}:10]")
            print(
                f"Baseline[{baseline_index}]           = Antenna[ant1] {cc.metafits_context.antennas[ant1].tile_id}, {cc.metafits_context.antennas[ant1].tile_name} vs Antenna[ant2] {cc.metafits_context.antennas[ant2].tile_id}, {cc.metafits_context.antennas[ant2].tile_name}")

            data_bl_index = baseline_index * (
                        cc.metafits_context.num_corr_fine_chans_per_coarse * cc.metafits_context.num_visibility_pols * 2)

            for chan in range(fine_chan_index, fine_chan_index + fine_chan_count):
                data_fine_index = data_bl_index + (chan * cc.metafits_context.num_visibility_pols * 2)
                print(f"chan {chan} "
                      f"XX: {data[data_fine_index]:.2f} {data[data_fine_index + 1]:.2f},\t"
                      f"XY: {data[data_fine_index + 2]:.2f} {data[data_fine_index + 3]:.2f},\t"
                      f"YX: {data[data_fine_index + 4]:.2f} {data[data_fine_index + 5]:.2f},\t"
                      f"YY: {data[data_fine_index + 6]:.2f} {data[data_fine_index + 7]:.2f}")
        else:
            with open(out_filename, "w") as out_file:
                for baseline_index in range(0, int(128*129/2)):
                    data_bl_index = baseline_index * (
                        cc.metafits_context.num_corr_fine_chans_per_coarse * cc.num_visibility_pols * 2)

                    for chan in range(fine_chan_index, fine_chan_index + fine_chan_count):
                        data_fine_index = data_bl_index + (chan * cc.num_visibility_pols * 2)

                        out_file.write(f"{data[data_fine_index]},{data[data_fine_index + 1]},"
                                       f"{data[data_fine_index + 2]},{data[data_fine_index + 3]},"
                                       f"{data[data_fine_index + 4]},{data[data_fine_index + 5]},"
                                       f"{data[data_fine_index + 6]},{data[data_fine_index + 7]}\n")
            print(f"Wrote {out_filename}")

def dump_pyuvdata(ant1, ant2, timestep_index, fine_chan_index, fine_chan_count, gpuboxfiles, metafits, out_filename):
    print("pyuvdata:")
    print("======================================")
    UV = UVData()
    file_list = gpuboxfiles + [metafits]

    UV.read_mwa_corr_fits(file_list, correct_cable_len=False, phase_to_pointing_center=False, flag_init=False,
                          remove_dig_gains=False, remove_coarse_band=False)
    UV.reorder_pols("CASA")

    # (1,128,4)
    if out_filename is None:
        data = UV.get_data(ant1, ant2)

        print(f"{UV.antenna_names[ant1]} vs {UV.antenna_names[ant2]}")

        for chan in range(fine_chan_index, fine_chan_index + fine_chan_count):
            print(f"chan {chan} "
                  f"XX: {data[timestep_index, chan, 0].real:.2f} {data[timestep_index, chan, 0].imag:.2f},\t"
                  f"XY: {data[timestep_index, chan, 1].real:.2f} {data[timestep_index, chan, 1].imag:.2f},\t"
                  f"YX: {data[timestep_index, chan, 2].real:.2f} {data[timestep_index, chan, 2].imag:.2f},\t"
                  f"YY: {data[timestep_index, chan, 3].real:.2f} {data[timestep_index, chan, 3].imag:.2f}")
    else:
        with open(out_filename, "w") as out_file:
            for a1 in range(0, 128):
                for a2 in range(a1, 128):
                    data = UV.get_data(a1, a2)

                    for chan in range(fine_chan_index, fine_chan_index + fine_chan_count):
                        out_file.write(f"{data[timestep_index, chan, 0].real},{data[timestep_index, chan, 0].imag},"
                                       f"{data[timestep_index, chan, 1].real},{data[timestep_index, chan, 1].imag},"
                                       f"{data[timestep_index, chan, 2].real},{data[timestep_index, chan, 2].imag},"
                                       f"{data[timestep_index, chan, 3].real},{data[timestep_index, chan, 3].imag}\n")

        print(f"Wrote {out_filename}")

def dump_casa(ant1, ant2, timestep_index, fine_chan_index, fine_chan_count, ms_filename, out_filename):
    print("casa:")
    print("======================================")

    t = casacore.tables.table(ms_filename)
    tr = t.row(["ANTENNA1", "ANTENNA2", "DATA"])

    # Present the data
    if out_filename is None:
        bl_index = get_baseline_from_antennas(ant1, ant2, 128)
        # Rows in the ms include baseline AND time. So we need to move down the table by n_bls x timestep
        n_bls = (128 * 129) / 2
        row_index = int(n_bls * timestep_index) + bl_index
        print(f"Antenna 1: {tr[row_index]['ANTENNA1']} Antenna 2: {tr[row_index]['ANTENNA2']}")  # get row info

        data = tr[row_index]['DATA']

        for chan in range(fine_chan_index, fine_chan_index + fine_chan_count):
            print(f"chan {chan} "
                  f"XX: {data[chan, 0].real:.2f} {data[chan, 0].imag:.2f},\t"
                  f"XY: {data[chan, 1].real:.2f} {data[chan, 1].imag:.2f},\t"
                  f"YX: {data[chan, 2].real:.2f} {data[chan, 2].imag:.2f},\t"
                  f"YY: {data[chan, 3].real:.2f} {data[chan, 3].imag:.2f}")
    else:
        with open(out_filename, "w") as out_file:
            # Rows in the ms include baseline AND time. So we need to move down the table by n_bls x timestep
            n_bls = int((128 * 129) / 2)

            for baseline_index in range(0, n_bls):
                row_index = int(n_bls * timestep_index) + baseline_index

                data = tr[row_index]['DATA']
                for chan in range(fine_chan_index, fine_chan_index + fine_chan_count):
                    out_file.write(f"{data[chan, 0].real},{data[chan, 0].imag},"
                                   f"{data[chan, 1].real},{data[chan, 1].imag},"
                                   f"{data[chan, 2].real},{data[chan, 2].imag},"
                                   f"{data[chan, 3].real},{data[chan, 3].imag}\n")

        print(f"Wrote {out_filename}")


def compare_lines(line1, line2):
    # Each line is a set of floats, comma separated
    # xx_r,xx_i,xy_r,xy_i,yx_r,yx_i,yy_r,yy_i
    floats1 = line1.split(",")
    floats2 = line2.split(",")

    mismatch = False

    for i, f in enumerate(floats1):
        if abs(float(f) - float(floats2[i])) > 0.01:
            mismatch = True

    return mismatch

def compare_csv(filename1, filename2):
    print(f"Comparing {filename1} with {filename2}")
    with open(filename1, "r") as file1:
        with open(filename2, "r") as file2:
            line_no = 1
            num_bl = int(128*129/2)
            num_fine_chans = 32

            while line_no < num_bl * num_fine_chans:
                line1 = file1.readline()
                line2 = file2.readline()

                if compare_lines(line1, line2):
                    print(f"Line: {line_no} mismatch:\n>>{line1}<<{line2}")
                line_no += 1

    print(f"Finished comparing {filename1} with {filename2}")


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("-m", "--metafits", required=True,
                        help="Path to an MWA metafits file.")
    parser.add_argument("gpuboxes", nargs='*',
                        help="Paths Legacy MWA gpubox files.")
    parser.add_argument("-c", "--casa-ms", required=False,
                        help="Path to the cotter generated CASA measurement set dir.")
    parser.add_argument("-o", "--console-output", required=False, help="If specified, will output to console only (not to file)", action='store_true')
    args = parser.parse_args()

    # dump the following baseline and fine channel for the timestep
    coarse_chan_index = 0
    timestep_index = 0
    ant1 = 0
    ant2 = 0
    baseline_index = get_baseline_from_antennas(ant1, ant2, 128)
    fine_chan_index = 0
    fine_chan_count = 128  # 10kHz obs=128, 20kHz=64, 40kHz=32

    dump_mwalib(ant1, ant2, timestep_index, fine_chan_index, fine_chan_count, args.gpuboxes, args.metafits, None if args.console_output else "mwalib.csv")
    dump_pyuvdata(ant1, ant2, timestep_index, fine_chan_index, fine_chan_count, args.gpuboxes, args.metafits, None if args.console_output else "pyuvdata.csv")

    if args.casa_ms:
        dump_casa(ant1, ant2, timestep_index, fine_chan_index, fine_chan_count, args.casa_ms, None if args.console_output else "cotter.csv")
    else:
        print("No cotter input file provided, so no cotter comparison will be produced.")
