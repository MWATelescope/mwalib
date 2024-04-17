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
import mwalib


def sum_parallel_by_bl(
    metafits_filename: str,
    gpubox_filenames: list,
    coarse_chan_index: int,
) -> float:
    chan_sum = 0.0

    with mwalib.CorrelatorContext(metafits_filename, gpubox_filenames) as context:
        if coarse_chan_index < context.num_coarse_chans:
            print(
                f"sum_parallel_by_bl: Summing {context.num_timesteps} timesteps "
                f"and coarse channel index {coarse_chan_index}..."
            )

            for t in range(0, context.num_timesteps):
                try:
                    data = context.read_by_baseline(t, coarse_chan_index)
                    chan_sum += np.sum(data, dtype=np.float64)

                except mwalib.GpuboxErrorNoDataForTimeStepCoarseChannel:
                    pass

                except Exception as e:
                    print(f"Error: {e}")
                    exit(-1)

    return chan_sum


def sum_parallel_by_freq(
    metafits_filename: str,
    gpubox_filenames: list,
    coarse_chan_index: int,
) -> float:
    chan_sum = 0.0

    with mwalib.CorrelatorContext(metafits_filename, gpubox_filenames) as context:
        if coarse_chan_index < context.num_coarse_chans:
            print(
                f"sum_parallel_by_freq: Summing {context.num_timesteps} timesteps "
                f"and coarse channel index {coarse_chan_index}..."
            )

            for t in range(0, context.num_timesteps):
                try:
                    data = context.read_by_frequency(t, coarse_chan_index)
                    chan_sum += np.sum(data, dtype=np.float64)

                except mwalib.GpuboxErrorNoDataForTimeStepCoarseChannel:
                    pass

                except Exception as e:
                    print(f"Error: {e}")
                    exit(-1)

    return chan_sum


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=f"Using mwalib {mwalib.__version__}")
    parser.add_argument(
        "-m", "--metafits", required=True, help="Path to the metafits file."
    )
    parser.add_argument("gpuboxes", nargs="*", help="Paths to the gpubox files.")
    args = parser.parse_args()

    # fast sum using all cores
    num_cores = os.cpu_count()
    print(
        f"Using {num_cores} cores to fast sum all hdus by baseline, then by"
        " frequency..."
    )

    start_time_fast = time.time()
    processed_list = Parallel(n_jobs=num_cores)(
        delayed(sum_parallel_by_bl)(args.metafits, args.gpuboxes, c) for c in range(24)
    )
    fast_sum = np.sum(processed_list)
    stop_time_fast = time.time()
    print(f"Sum is: {fast_sum} in {stop_time_fast - start_time_fast} seconds.\n")

    start_time_fast = time.time()
    processed_list = Parallel(n_jobs=num_cores)(
        delayed(sum_parallel_by_freq)(args.metafits, args.gpuboxes, c)
        for c in range(24)
    )
    fast_sum = np.sum(processed_list)
    stop_time_fast = time.time()
    print(f"Sum is: {fast_sum} in {stop_time_fast - start_time_fast} seconds.\n")
