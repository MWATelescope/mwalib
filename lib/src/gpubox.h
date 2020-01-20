#pragma once

#include <fitsio.h>

#include "global.h"

static const int PFB_MAP[64] = {0,  16, 32, 48, 1,  17, 33, 49, 2,  18, 34, 50, 3,  19, 35, 51,
                                4,  20, 36, 52, 5,  21, 37, 53, 6,  22, 38, 54, 7,  23, 39, 55,
                                8,  24, 40, 56, 9,  25, 41, 57, 10, 26, 42, 58, 11, 27, 43, 59,
                                12, 28, 44, 60, 13, 29, 45, 61, 14, 30, 46, 62, 15, 31, 47, 63};

typedef struct mwaObsContext {
    long obsid;

    long long start_time_milliseconds;
    long long end_time_milliseconds;

    // num_integrations only considers data between the start and end times!
    long num_integrations;
    long num_baselines;
    int num_pols;

    int num_fine_channels;
    int num_coarse_channels;
    int *coarse_channels;

    char *metafits_filename;
    fitsfile *metafits_ptr;

    int gpubox_filename_count;
    // Elements of gpubox_filenames are expected to be in the same order as
    // gpubox_ptrs. This concept applies all over mwalib.
    char **gpubox_filenames;
    fitsfile **gpubox_ptrs;
    // gpubox batches refers to the different gpubox outputs for the same course
    // band channel. e.g. "1065880128_20131015134830_gpubox01_00.fits" belongs
    // to "batch 0", whereas "1065880128_20131015134930_gpubox01_01.fits"
    // belongs to "batch 1".
    int gpubox_batch_count;
    // The following container pointers to gpubox_filenames and gpubox_ptrs, and
    // are structured e.g. gpubox_filename_batches[batch][0] = pointer to first
    // filename.
    char ****gpubox_filename_batches;
    fitsfile ****gpubox_ptr_batches;
} mwaObsContext_s;

int free_mwaObsContext(mwaObsContext_s *obs);

int determine_gpubox_fine_channels(mwaObsContext_s *obs, char *errorMessage);
int determine_gpubox_batches(mwaObsContext_s *obs, char *errorMessage);
int determine_obs_times(mwaObsContext_s *obs, char *errorMessage);
