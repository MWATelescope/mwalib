/**
 * @file args.c
 * @author Greg Sleap
 * @date 18 Dec 2019
 * @brief This is the code that parses and validates command line arguments
 * passed in from client program
 *
 */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "args.h"
#include "fitsreader.h"
#include "global.h"
#include "gpubox.h"

/**
 *
 *  @brief This function initialises all of the members of the args struct
 *  @param[in] args Pointer to the mwalibArgs_s structure where we put the
 * parsed arguments.
 *  @returns EXIT_SUCCESS on success, or -1 if there was an error.
 */
int initialise_args(mwalibArgs_s *args)
{
    args->gpubox_filename_count = 0;

    return EXIT_SUCCESS;
}

/**
 *
 *  @brief This function sets the metafits filename in the args struct and
 * allocates a buffer for it
 *  @param[in] args Pointer to the mwalibArgs_s structure where we put the
 * parsed arguments.
 *  @param[in] filename Pointer to the filename
 *  @returns EXIT_SUCCESS on success, or -1 if there was an error.
 */
int set_metafits_filename(mwalibArgs_s *args, char *filename)
{
    // Set file to our struct
    args->metafits_filename = (char *)malloc(sizeof(char) * (strlen(filename) + 1));
    if (args->metafits_filename == NULL) {
        // This function currently only returns EXIT_FAILURE in one spot, so we
        // can assume that if this function ever fails, it's because malloc
        // failed.
        return EXIT_FAILURE;
    }
    strcpy(args->metafits_filename, filename);
    return EXIT_SUCCESS;
}

/**
 *
 *  @brief This function adds a new gpubox filename in the args struct,
 * allocates a buffer for it and increments the gpubox counter
 *  @param[in] args Pointer to the mwalibArgs_s structure where we put the
 * parsed arguments.
 *  @param[in] filename Pointer to the filename
 *  @returns EXIT_SUCCESS on success, or -1 if there was an error.
 */
int add_gpubox_filename(mwalibArgs_s *args, char *filename)
{
    // Set file to our struct
    args->gpubox_filenames[args->gpubox_filename_count] = (char *)malloc(sizeof(char) * (strlen(filename) + 1));
    if (args->gpubox_filenames[args->gpubox_filename_count] == NULL) {
        // This function currently only returns EXIT_FAILURE in one spot, so we
        // can assume that if this function ever fails, it's because malloc
        // failed.
        return EXIT_FAILURE;
    }
    strcpy(args->gpubox_filenames[args->gpubox_filename_count], filename);
    args->gpubox_filename_count = args->gpubox_filename_count + 1;

    return EXIT_SUCCESS;
}

/**
 * @brief This function validates the mwalibArgs_s passed in, and populates a
 * mwaObsContext_s with it. Returns success if all good.
 * @param[in] args Pointer to the mwalibArgs_s structure where we put the parsed
 * arguments.
 * @param[inout] obs Pointer to the mwaObsContext_s to be populated.
 * @param[inout] errorMessage Pointer to a string of length
 * MWALIB_ERROR_MESSAGE_LEN containing an error message or empty string if no
 * error.
 * @returns EXIT_SUCCESS on success, or EXIT_FAILURE if there was an error.
 */
int process_args(mwalibArgs_s *args, mwaObsContext_s *obs, char *errorMessage)
{
    // Initialise errorMessage
    strncpy(errorMessage, "", MWALIB_ERROR_MESSAGE_LEN);

    // Check for prescence of metafits
    if (strlen(args->metafits_filename) == 0) {
        snprintf(errorMessage, MWALIB_ERROR_MESSAGE_LEN, "Metafits filename missing");
        return EXIT_FAILURE;
    }

    // Check for prescence of any gpubox files
    if (args->gpubox_filename_count == 0) {
        snprintf(errorMessage, MWALIB_ERROR_MESSAGE_LEN, "gpubox / mwax fits files missing");
        return EXIT_FAILURE;
    }

    // Open the metafits file
    obs->metafits_filename = args->metafits_filename;
    if (open_fits(&(obs->metafits_ptr), obs->metafits_filename, errorMessage) != EXIT_SUCCESS) {
        return EXIT_FAILURE;
    }

    // Get the OBSID
    if (get_fits_int_value(obs->metafits_ptr, "GPSTIME", &(obs->obsid), errorMessage) != EXIT_SUCCESS) {
        return EXIT_FAILURE;
    }

    // Always assume that MWA data has four polarisations. Would this ever not
    // be true?
    obs->num_pols = 4;

    // Calculate the number of baselines. There are twice as many inputs as
    // there are antennas; halve that value.
    int num_inputs = 0;
    if (get_fits_int_value(obs->metafits_ptr, "NINPUTS", &num_inputs, errorMessage) != EXIT_SUCCESS) {
        return EXIT_FAILURE;
    }
    num_inputs /= 2;
    obs->num_baselines = num_inputs / 2 * (num_inputs - 1);

    // CHANNELS
    char coarse_channel_string[1024] = {""};
    if (get_fits_string_value(obs->metafits_ptr, "CHANNELS", coarse_channel_string, errorMessage) != EXIT_SUCCESS) {
        return EXIT_FAILURE;
    }
    printf("Coarse channels = %s\n", coarse_channel_string);

    // Open the gpubox files.
    obs->gpubox_filename_count = args->gpubox_filename_count;
    // Allocate the obs struct's gpubox_filenames and gpubox_ptrs members.
    obs->gpubox_filenames = (char **)malloc(sizeof(char *) * obs->gpubox_filename_count);
    obs->gpubox_ptrs = (fitsfile **)malloc(sizeof(fitsfile *) * obs->gpubox_filename_count);
    if (obs->gpubox_filenames == NULL) {
        snprintf(errorMessage, MWALIB_ERROR_MESSAGE_LEN, "malloc failed for obs->gpubox_filenames");
        return EXIT_FAILURE;
    }
    if (obs->gpubox_ptrs == NULL) {
        snprintf(errorMessage, MWALIB_ERROR_MESSAGE_LEN, "malloc failed for obs->gpubox_ptrs");
        return EXIT_FAILURE;
    }

    for (int i = 0; i < obs->gpubox_filename_count; i++) {
        // Copy the gpubox filename from the args struct to the obs struct.
        obs->gpubox_filenames[i] = (char *)malloc(sizeof(char) * MWALIB_MAX_GPUBOX_FILENAME_LEN);
        if (obs->gpubox_filenames[i] == NULL) {
            snprintf(errorMessage, MWALIB_ERROR_MESSAGE_LEN, "malloc failed for obs->gpubox_filenames[i], i = %d", i);
            return EXIT_FAILURE;
        }
        obs->gpubox_filenames[i] = args->gpubox_filenames[i];

        if (open_fits(&obs->gpubox_ptrs[i], obs->gpubox_filenames[i], errorMessage) != EXIT_SUCCESS) {
            return EXIT_FAILURE;
        }
    }

    // Populate the fine channels. For some reason, this isn't in the metafits.
    if (determine_gpubox_fine_channels(obs, errorMessage) != EXIT_SUCCESS) {
        sprintf(errorMessage + strlen(errorMessage), " (determine_gpubox_fine_channels)");
        return EXIT_FAILURE;
    }

    // Populate the gpubox batches.
    if (determine_gpubox_batches(obs, errorMessage) != EXIT_SUCCESS) {
        sprintf(errorMessage + strlen(errorMessage), " (determine_gpubox_batches)");
        return EXIT_FAILURE;
    }

    // Populate the start and end times of the observation.
    if (determine_obs_times(obs, errorMessage) != EXIT_SUCCESS) {
        sprintf(errorMessage + strlen(errorMessage), " (determine_obs_times)");
        return EXIT_FAILURE;
    }

    return EXIT_SUCCESS;
}
