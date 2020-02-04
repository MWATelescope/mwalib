#!/usr/bin/env python

# Adapted from:
# http://jakegoulding.com/rust-ffi-omnibus/objects/

# Additional documentation:
# https://docs.python.org/3.8/library/ctypes.html#module-ctypes

import sys
import argparse
import ctypes as ct


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

mwalib.mwalibContext_free.argtypes = (ct.POINTER(MwalibContextS), )

mwalib.mwalibContext_display.argtypes = (ct.POINTER(MwalibContextS), )


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

    def display(self):
        mwalib.mwalibContext_display(self.obj)


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
        context.display()
