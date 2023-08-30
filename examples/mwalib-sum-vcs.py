#!/usr/bin/env python
#
# pymwalib examples/sum-vcs - utilise all cores to sum the vcs data files and compare against single threaded
#
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.
#
# NOTE: this example requires numpy and joblib packages. These can be installed via pip.
# e.g. pip install numpy
#      pip install joblib
#
import argparse
import time
import numpy as np
import mwalib


def sum_by_file(
    context: mwalib.VoltageContext, timestep_index: int, coarse_chan_index: int
) -> int:
    total_sum = 0

    start_gpstime_sec = int(context.timesteps[t].gps_time_ms / 1000)
    gps_seconds_count = int(context.timestep_duration_ms / 1000)
    end_gpstime_sec = start_gpstime_sec + gps_seconds_count

    print(
        (
            f"...Summing timestep {timestep_index} {start_gpstime_sec} to"
            f" {end_gpstime_sec} ({gps_seconds_count} seconds) and chan:"
            f" {coarse_chan_index}..."
        ),
        end="",
    )

    try:
        data = context.read_file(timestep_index, coarse_chan_index)
        total_sum = data.sum(dtype=np.int64)
        print(f"{total_sum}")

    except mwalib.VoltageErrorNoDataForTimeStepCoarseChannel:
        pass

    except Exception as e:
        print(f"Error: {e}")
        exit(-1)

    return total_sum


def sum_by_gps_second(
    context: mwalib.VoltageContext,
    gps_start_sec,
    gps_end_sec,
    gps_seconds_count,
    coarse_chan_index: int,
) -> int:
    total_sum = 0

    print(
        (
            f"...Summing {gps_start_sec} to {gps_end_sec} ({gps_seconds_count} seconds)"
            f" and chan: {coarse_chan_index}..."
        ),
        end="",
    )
    try:
        data = context.read_second(gps_start_sec, gps_seconds_count, coarse_chan_index)
        total_sum = data.sum(dtype=np.int64)
        print(f"{total_sum}")

    except mwalib.VoltageErrorNoDataForTimeStepCoarseChannel:
        pass

    except Exception as e:
        print(f"Error: {e}")
        exit(-1)

    return total_sum


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=f"Using mwalib {mwalib.__version__}")
    parser.add_argument(
        "-m", "--metafits", required=True, help="Path to the metafits file."
    )
    parser.add_argument("datafiles", nargs="*", help="Paths to the vcs data files.")
    args = parser.parse_args()

    context = mwalib.VoltageContext(args.metafits, args.datafiles)

    # sum by file
    print(
        f"sum_by_file: Summing {context.num_provided_timesteps} timesteps "
        f"and {context.num_provided_coarse_chans} coarse channels..."
    )
    total_sum_by_file = 0
    start_time = time.time()
    for t in context.provided_timestep_indices:
        for c in context.provided_coarse_chan_indices:
            total_sum_by_file += np.sum(sum_by_file(context, t, c))
    stop_time = time.time()
    print(f"Sum is: {total_sum_by_file} in {stop_time - start_time} seconds.\n")

    # sum by gps second
    start_gpstime_sec = int(
        context.timesteps[context.provided_timestep_indices[0]].gps_time_ms / 1000
    )
    end_gpstime_sec = int(
        (
            context.timesteps[
                context.provided_timestep_indices[context.num_provided_timesteps - 1]
            ].gps_time_ms
            + context.timestep_duration_ms
        )
        / 1000
    )
    gps_second_count = end_gpstime_sec - start_gpstime_sec

    print(
        f"sum_by_gps_second: Summing {context.num_provided_timesteps} timesteps and"
        f" {context.num_provided_coarse_chans} coarse channels... (gps_seconds:"
        f" {gps_second_count})"
    )
    total_sum_by_gps = 0
    start_time = time.time()
    for c in context.provided_coarse_chan_indices:
        total_sum_by_gps += sum_by_gps_second(
            context, start_gpstime_sec, end_gpstime_sec, gps_second_count, c
        )
    stop_time = time.time()
    print(f"Sum is: {total_sum_by_gps} in {stop_time - start_time} seconds.")
