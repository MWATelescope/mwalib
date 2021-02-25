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
mwalib.mwalib_correlator_context_new.restype = ct.c_int32

mwalib.mwalib_correlator_context_free.argtypes = (ct.POINTER(CorrelatorContextS), )

mwalib.mwalib_correlator_context_read_by_baseline.argtypes = \
    (ct.POINTER(CorrelatorContextS), # context
     ct.c_size_t,                # input timestep_index
     ct.c_size_t,                # input coarse_chan_index
     ct.POINTER(ct.c_float),     # buffer_ptr
     ct.c_size_t,                # buffer_len
     ct.c_char_p,                # error message
     ct.c_size_t)                # length of error message
mwalib.mwalib_correlator_context_read_by_baseline.restype = ct.c_int32

mwalib.mwalib_correlator_context_read_by_frequency.argtypes = \
    (ct.POINTER(CorrelatorContextS), # context
     ct.c_size_t,                # input timestep_index
     ct.c_size_t,                # input coarse_chan_index
     ct.POINTER(ct.c_float),     # buffer_ptr
     ct.c_size_t,                # buffer_len
     ct.c_char_p,               # error message
     ct.c_size_t)               # length of error message
mwalib.mwalib_correlator_context_read_by_frequency.restype = ct.c_int32


class MWAlibContext:
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

        # for now we will hard code this
        # TODO fix this once we have metadata population
        self.num_timesteps = 1
        self.num_floats = 8256 * 128 * 4 * 2

    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc_value, traceback):
        mwalib.mwalib_correlator_context_free(self.correlator_context)

    def read_by_baseline(self, timestep_index, coarse_chan_index):
        error_message = " ".encode("utf-8") * ERROR_MESSAGE_LEN

        float_buffer_type = ct.c_float * self.num_floats
        buffer = float_buffer_type()

        if mwalib.mwalib_correlator_context_read_by_baseline(self.correlator_context, ct.c_size_t(timestep_index),
                                                 ct.c_size_t(coarse_chan_index),
                                                 buffer, self.num_floats,
                                                 error_message, ERROR_MESSAGE_LEN) != 0:
            raise Exception(f"Error reading data: {error_message.decode('utf-8').rstrip()}")
        else:
            return npct.as_array(buffer, shape=(self.num_floats,))

    def read_by_frequency(self, timestep_index, coarse_chan_index):
        error_message = " ".encode("utf-8") * ERROR_MESSAGE_LEN

        float_buffer_type = ct.c_float * self.num_floats
        buffer = float_buffer_type()

        if mwalib.mwalib_correlator_context_read_by_baseline(self.correlator_context, ct.c_size_t(timestep_index),
                                                 ct.c_size_t(coarse_chan_index),
                                                 buffer, self.num_floats,
                                                 error_message, ERROR_MESSAGE_LEN) != 0:
            raise Exception(f"Error reading data: {error_message.decode('utf-8').rstrip()}")
        else:
            return npct.as_array(buffer, shape=(self.num_floats,))


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("-b", "--sum-by-bl", help="Sum by baseline.", action="store_true")
    parser.add_argument("-f", "--sum-by-freq", help = "Sum by freq.", action="store_true")
    parser.add_argument("-m", "--metafits", required=True,
                        help="Path to the metafits file.")
    parser.add_argument("gpuboxes", nargs='*',
                        help="Paths to the gpubox files.")
    args = parser.parse_args()

    with MWAlibContext(args.metafits, args.gpuboxes) as context:
        num_timesteps = context.num_timesteps
        num_coarse_chans = 1
        num_fine_chans = 128
        num_baselines = 8256
        num_vis_pols = 4
        #num_floats = num_baselines * \
        #    num_fine_chans * num_vis_pols * 2

        sum = 0.0

        if args.sum_by_bl:
            sum = 0
            print("Summing by baseline...")
            for timestep_index in range(0, num_timesteps):
                this_sum = 0

                for coarse_chan_index in range(0, num_coarse_chans):
                    try:
                        data = context.read_by_baseline(timestep_index,
                                                        coarse_chan_index)
                    except Exception as e:
                        print(f"Error: {e}")
                        exit(-1)

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
                    this_sum = np.sum(data,
                                      dtype=np.float64)

                    sum += this_sum
            print("Total sum: {}".format(sum))

        if args.sum_by_freq:
            sum = 0
            print("Summing by frequency...")

            for timestep_index in range(0, num_timesteps):
                this_sum = 0

                for coarse_chan_index in range(0, num_coarse_chans):
                    try:
                        data = context.read_by_frequency(timestep_index,
                                                         coarse_chan_index)
                    except Exception as e:
                        print(f"Error: {e}")
                        exit(-1)

                    this_sum = np.sum(data,
                                      dtype=np.float64)

                    sum += this_sum

            print("Total sum: {}".format(sum))
