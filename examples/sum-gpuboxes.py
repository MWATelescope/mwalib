#!/usr/bin/env python
#
# pymwalib examples/sum-gpuboxes - utilise all cores to sum the hdus and compare against single threaded
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
import os
import time

import numpy as np
from joblib import Parallel, delayed

from pymwalib.correlator_context import CorrelatorContext
from pymwalib.errors import PymwalibNoDataForTimestepAndCoarseChannelError
from pymwalib.version import check_mwalib_version


def sum_by_baseline_task(metafits_filename: str, gpubox_filenames: list, coarse_chan_index: int) -> float:
    chan_sum = 0.

    with CorrelatorContext(metafits_filename, gpubox_filenames) as context:
        if coarse_chan_index < context.num_coarse_chans:
            print(f"sum_by_baseline_task: Summing {context.num_timesteps} timesteps "
                  f"and coarse channel index {coarse_chan_index}...")

            for t in range(0, context.num_timesteps):
                try:
                    data = context.read_by_frequency(t, coarse_chan_index)
                    chan_sum += np.sum(data, dtype=np.float64)

                except PymwalibNoDataForTimestepAndCoarseChannelError:
                    pass

                except Exception as e:
                    print(f"Error: {e}")
                    exit(-1)

    return chan_sum


def sum_by_baseline_slow(metafits_filename: str, gpubox_filenames: list) -> float:
    total_sum = 0.

    with CorrelatorContext(metafits_filename, gpubox_filenames) as context:
        for coarse_chan_index in range(0, context.num_coarse_chans):
            print(f"sum_by_baseline_slow: Summing {context.num_timesteps} timesteps "
                  f"and coarse channel index {coarse_chan_index}...")
            for timestep_index in range(0, context.num_timesteps):
                try:
                    data = context.read_by_baseline(timestep_index,
                                                    coarse_chan_index)
                    total_sum += np.sum(data, dtype=np.float64)

                except PymwalibNoDataForTimestepAndCoarseChannelError:
                    pass

                except Exception as e:
                    print(f"Error: {e}")
                    exit(-1)

    return total_sum


if __name__ == "__main__":
    # ensure we have a compatible mwalib first
    # You can skip this if you want, but your first pymwalib call will raise an error. Best trap it here
    # and provide a nice user message
    try:
        check_mwalib_version()
    except Exception as e:
        print(e)
        exit(1)

    parser = argparse.ArgumentParser()
    parser.add_argument("-m", "--metafits", required=True,
                        help="Path to the metafits file.")
    parser.add_argument("gpuboxes", nargs='*',
                        help="Paths to the gpubox files.")
    args = parser.parse_args()

    # fast sum using all cores
    num_cores = len(os.sched_getaffinity(0))
    print(f"Using {num_cores} cores to fast sum all hdus...")

    start_time_fast = time.time()
    processed_list = Parallel(n_jobs=num_cores)(
        delayed(sum_by_baseline_task)(args.metafits, args.gpuboxes, c) for c in range(24))
    fast_sum = np.sum(processed_list)
    stop_time_fast = time.time()
    print(
        f"Sum is: {fast_sum} in {stop_time_fast - start_time_fast} seconds.\n")

    # slow sum restricted to one python process
    start_time_slow = time.time()
    slow_sum = sum_by_baseline_slow(args.metafits, args.gpuboxes)
    stop_time_slow = time.time()
    print(f"Sum is: {slow_sum} in {stop_time_slow - start_time_slow} seconds.")
