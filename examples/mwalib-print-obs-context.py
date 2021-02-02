#!/usr/bin/env python

# Adapted from:
# http://jakegoulding.com/rust-ffi-omnibus/objects/

# Additional documentation:
# https://docs.python.org/3.8/library/ctypes.html#module-ctypes

import sys
import argparse
import ctypes as ct

ERROR_MESSAGE_LEN = 1024


class CorrelatorContextS(ct.Structure):
    pass


prefix = {"win32": ""}.get(sys.platform, "lib")
extension = {"darwin": ".dylib", "win32": ".dll"}.get(sys.platform, ".so")
mwalib_filename = prefix + "mwalib" + extension
mwalib = ct.cdll.LoadLibrary(mwalib_filename)

mwalib.mwalib_correlator_context_new.argtypes = \
    (ct.c_char_p,              # metafits
     ct.POINTER(ct.c_char_p),  # gpuboxes
     ct.c_size_t,              # gpubox count
     ct.POINTER(ct.POINTER(CorrelatorContextS)), # Pointer to pointer to CorrelatorContext
     ct.c_char_p,              # error message
     ct.c_size_t)              # length of error message
mwalib.mwalib_correlator_context_new.restype = ct.c_uint32

mwalib.mwalib_correlator_context_free.argtypes = (ct.POINTER(CorrelatorContextS), )

mwalib.mwalib_correlator_context_display.argtypes = (ct.POINTER(CorrelatorContextS), )
mwalib.mwalib_correlator_context_display.restype = ct.c_uint32


class CorrelatorContext:
    def __init__(self, metafits, gpuboxes):
        # Encode all inputs as UTF-8.
        m = ct.c_char_p(metafits.encode("utf-8"))

        # https://stackoverflow.com/questions/4145775/how-do-i-convert-a-python-list-into-a-c-array-by-using-ctypes
        encoded = []
        for g in gpuboxes:
            encoded.append(ct.c_char_p(g.encode("utf-8")))
        seq = ct.c_char_p * len(encoded)
        g = seq(*encoded)
        error_message: bytes = " ".encode("utf-8") * ERROR_MESSAGE_LEN

        self.correlator_context = ct.POINTER(CorrelatorContextS)()

        if mwalib.mwalib_correlator_context_new(m, g, len(encoded), ct.byref(self.correlator_context), error_message, ERROR_MESSAGE_LEN) != 0:
            print(f"Error creating context: {error_message.decode('utf-8').rstrip()}")

    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc_value, traceback):
        mwalib.mwalib_correlator_context_free(self.correlator_context)

    def display(self):
        error_message = " ".encode("utf-8") * ERROR_MESSAGE_LEN

        if mwalib.mwalib_correlator_context_display(self.correlator_context, error_message, ERROR_MESSAGE_LEN) != 0:
            print(
                f"Error calling mwalib_correlator_context_display(): {error_message.decode('utf-8').rstrip()}")
            exit(1)


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("-m", "--metafits", required=True,
                        help="Path to the metafits file.")
    parser.add_argument("gpuboxes", nargs='*',
                        help="Paths to the gpubox files.")
    args = parser.parse_args()

    with CorrelatorContext(args.metafits, args.gpuboxes) as context:
        context.display()
