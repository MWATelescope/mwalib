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

mwalib.mwalibContext_read.argtypes = \
    (ct.POINTER(MwalibContextS),
     ct.POINTER(ct.c_int),       # input/output scan count
     ct.POINTER(ct.c_int),       # output gpubox count
     ct.POINTER(ct.c_longlong))  # output HDU size
mwalib.mwalibContext_read.restype = \
    ct.POINTER(ct.POINTER(ct.POINTER(ct.c_float)))

mwalib.free_float_buffer.argtypes = \
    (ct.POINTER(ct.POINTER(ct.POINTER(ct.c_float))),)


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

    def read(self, num_scans, num_gpubox_files, gpubox_hdu_size):
        return mwalib.mwalibContext_read(self.obj, num_scans,
                                         num_gpubox_files, gpubox_hdu_size)


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("-n", "--num-scans", default=3, type=int,
                        help="Number of MWA scans to read in a time. Default: %(default)s.")
    parser.add_argument("-m", "--metafits", required=True,
                        help="Path to the metafits file.")
    parser.add_argument("gpuboxes", nargs='*',
                        help="Paths to the gpubox files.")
    args = parser.parse_args()

    with MwalibContext(args.metafits, args.gpuboxes) as context:
        num_scans = ct.c_int(args.num_scans)
        num_gpubox_files = ct.c_int(0)
        gpubox_hdu_size = ct.c_longlong(0)

        sum = 0.0
        while num_scans.value > 0:
            data = context.read(ct.byref(num_scans),
                                ct.byref(num_gpubox_files),
                                ct.byref(gpubox_hdu_size))

            # "data" is just a nested array of pointers at the moment. If one
            # wanted to create and populate a numpy array with the raw MWA data,
            # then the following would work.
            # np_data = np.empty((num_scans.value,
            #                     num_gpubox_files.value,
            #                     gpubox_hdu_size.value),
            #                    dtype=np.float32)
            # for s in range(num_scans.value):
            #     for g in range(num_gpubox_files.value):
            #         np_data[s][g] = npct.as_array(data[s][g], shape=(gpubox_hdu_size.value,))

            # But, in this example, we're only interested in adding all the data
            # into a single number.
            for s in range(num_scans.value):
                for g in range(num_gpubox_files.value):
                    sum += np.sum(npct.as_array(data[s][g], shape=(gpubox_hdu_size.value,)),
                                  dtype=np.float64)

            # Free the memory via rust (python can't do it).
            if num_scans.value > 0:
                mwalib.free_float_buffer(data,
                                         ct.byref(num_scans),
                                         ct.byref(num_gpubox_files),
                                         ct.byref(gpubox_hdu_size))

    print("Total sum:", sum)
