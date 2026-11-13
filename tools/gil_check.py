"""Times reads from 1 and 4 threads. With the GIL released, 4 threads take about the same time per read on >= 4 cores."""

import time
from concurrent.futures import ThreadPoolExecutor

import mwalib

META = "test_files/1101503312_1_timestep/1101503312.metafits"
GPUBOX = ["test_files/1101503312_1_timestep/1101503312_20141201210818_gpubox01_00.fits"]
NUM_READS = 16

cc = mwalib.CorrelatorContext(META, GPUBOX)
for workers in (1, 4):
    start = time.perf_counter()
    with ThreadPoolExecutor(max_workers=workers) as ex:
        list(ex.map(lambda _: cc.read_by_frequency(0, 0), range(NUM_READS)))
    print(f"{workers} thread(s): {time.perf_counter() - start:.3f} s for {NUM_READS} reads")
