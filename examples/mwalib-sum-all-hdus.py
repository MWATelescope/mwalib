#!/usr/bin/env python

# Given gpubox files, add their entire contents and report the sum.

# Adapted from:
# http://jakegoulding.com/rust-ffi-omnibus/objects/

# Additional documentation:
# https://docs.python.org/3.8/library/ctypes.html#module-ctypes

import sys
import argparse
import ctypes as ct
import numpy as np
import numpy.ctypeslib as npct


class MwalibContextS(ct.Structure):
    pass


prefix = {"win32": ""}.get(sys.platform, "lib")
extension = {"darwin": ".dylib", "win32": ".dll"}.get(sys.platform, ".so")
path_to_mwalib = "../target/release/" + prefix + "mwalib" + extension
mwalib = ct.cdll.LoadLibrary(path_to_mwalib)

mwalib.mwalibContext_new.argtypes = \
    (ct.c_char_p,              # metafits
     ct.POINTER(ct.c_char_p),  # gpuboxes
     ct.c_size_t)              # gpubox count
mwalib.mwalibContext_new.restype = ct.POINTER(MwalibContextS)

mwalib.mwalibContext_free.argtypes = (ct.POINTER(MwalibContextS),)

mwalib.mwalibContext_read_one_timestep_coarse_channel_bfp.argtypes = \
    (ct.POINTER(MwalibContextS),
     ct.POINTER(ct.c_int),       # input timestep_index
     ct.POINTER(ct.c_int))       # input coarse_channel_index
mwalib.mwalibContext_read_one_timestep_coarse_channel_bfp.restype = ct.POINTER(
    ct.c_float)

mwalib.free_float_buffer.argtypes = (
    ct.POINTER(ct.c_float),)


class MwalibContext:
    def __init__(self, metafits, gpuboxes):
        # Encode all inputs as UTF-8.
        m = ct.c_char_p(metafits.encode("utf-8"))

        # https://stackoverflow.com/questions/4145775/how-do-i-convert-a-python-list-into-a-c-array-by-using-ctypes
        encoded = []
        for g in gpuboxes:
            encoded.append(ct.c_char_p(g.encode("utf-8")))
        seq = ct.c_char_p * len(encoded)
        g = seq(*encoded)
        self.obj = mwalib.mwalibContext_new(m, g, len(encoded))

    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc_value, traceback):
        mwalib.mwalibContext_free(self.obj)

    def read_one_timestep_coarse_channel_bfp(self, timestep_index, coarse_channel_index):
        return mwalib.mwalibContext_read_one_timestep_coarse_channel_bfp(self.obj, timestep_index, coarse_channel_index)


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("-m", "--metafits", required=True,
                        help="Path to the metafits file.")
    parser.add_argument("gpuboxes", nargs='*',
                        help="Paths to the gpubox files.")
    args = parser.parse_args()

    with MwalibContext(args.metafits, args.gpuboxes) as context:
        num_timesteps = 53
        num_coarse_channels = 1
        num_fine_channels = 128
        num_baselines = 8256
        num_vis_pols = 4
        num_floats = num_baselines * \
            num_fine_channels * num_vis_pols * 2

        sum = 0.0
        for timestep_index in range(0, num_timesteps):
            this_sum = 0

            for coarse_channel_index in range(0, num_coarse_channels):
                data = context.read_one_timestep_coarse_channel_bfp(ct.byref(ct.c_int(timestep_index)),
                                                                    ct.byref(ct.c_int(coarse_channel_index)))

                # "data" is just an array of pointers at the moment. If one
                # wanted to create and populate a numpy array with the raw MWA data,
                # then the following would work.
                # np_data = np.empty(gpubox_hdu_size),
                #                    dtype=np.float32)
                # for s in range(num_scans.value):
                #     for g in range(num_gpubox_files.value):
                #         np_data[s][g] = npct.as_array(data[s][g], shape=(gpubox_hdu_size.value,))

                # But, in this example, we're only interested in adding all the data
                # into a single number.
                this_sum = np.sum(npct.as_array(data, shape=(num_floats,)),
                                  dtype=np.float64)

                sum += this_sum

                # Free the memory via rust (python can't do it).
                mwalib.free_float_buffer(data,
                                         ct.byref(ct.c_longlong(num_floats * 4)))

    print("Total sum: {}".format(sum))
