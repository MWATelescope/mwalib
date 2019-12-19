/**
 * @file args.h
 * @author Greg Sleap
 * @date 19 Dec 2019
 * @brief This is the header for code that parses and validates command line arguments passed in from client program
 *
 */
#pragma once

#include <fitsio.h>
#include <linux/limits.h>
#include "global.h"

// Args structure
typedef struct mwalibArgs
{        
    char* metafits_filename;
    fitsfile* metafits_ptr;

    char* gpubox_filenames[MWALIB_MAX_GPUBOX_FILENAMES];
    int gpubox_filename_count;

    int obs_id;
    int* coarse_channel_numbers[MWALIB_MAX_COARSE_CHANNELS];
} mwalibArgs_s;

int initialise_args(mwalibArgs_s* args);
int set_metafits_filename(mwalibArgs_s* args, char* filename);
int add_gpubox_filename(mwalibArgs_s* args, char* filename);
int process_args(mwalibArgs_s* args, char* errorMessage);