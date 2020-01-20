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
 * @brief This function initialises all of the members of the args struct. The
 * helper functions set_metafits_filename and add_gpubox_filename should be used
 * to fully populate the struct, after this function is called.
 * @param[in] args Pointer to the mwalibArgs_s structure where we put the
 * parsed arguments.
 * @returns EXIT_SUCCESS on success, or EXIT_FAILURE if there was an error.
 */
int initialise_args(mwalibArgs_s *args)
{
    args->metafits_filename = NULL;
    args->gpubox_filename_count = 0;
    args->gpubox_filenames = (char **)malloc(sizeof(char *) * MWALIB_MAX_GPUBOX_FILENAMES);
    if (args->gpubox_filenames == NULL) {
        // This function currently only returns EXIT_FAILURE in one spot, so we
        // can assume that if this function ever fails, it's because malloc
        // failed.
        return EXIT_FAILURE;
    }

    return EXIT_SUCCESS;
}

/**
 * @brief Deallocate the members of the args struct.
 * @param[in] args Pointer to the mwalibArgs_s structure whose members are to be
 * deallocated.
 * @returns EXIT_SUCCESS on success, or EXIT_FAILURE if there was an error.
 */
int free_args(mwalibArgs_s *args)
{
    free(args->metafits_filename);
    for (int i = 0; i < args->gpubox_filename_count; i++) {
        free(args->gpubox_filenames[i]);
    }
    free(args->gpubox_filenames);

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

    // Copy the metafits filename string to the obs struct.
    obs->metafits_filename = (char *)malloc(sizeof(char) * (strlen(args->metafits_filename) + 1));
    if (obs->metafits_filename == NULL) {
        // This function currently only returns EXIT_FAILURE in one spot, so we
        // can assume that if this function ever fails, it's because malloc
        // failed.
        return EXIT_FAILURE;
    }

    strcpy(obs->metafits_filename, args->metafits_filename);

    // Open the metafits file
    if (open_fits(&(obs->metafits_ptr), obs->metafits_filename, errorMessage) != EXIT_SUCCESS) {
        return EXIT_FAILURE;
    }

    // Get the OBSID
    if (get_fits_long_value(obs->metafits_ptr, "GPSTIME", &(obs->obsid), errorMessage) != EXIT_SUCCESS) {
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

    // Copy the gpubox filename strings to the obs struct, and open the files,
    // saving the file pointers.
    obs->gpubox_filename_count = args->gpubox_filename_count;
    obs->gpubox_filenames = (char **)malloc(sizeof(char *) * obs->gpubox_filename_count);
    if (obs->gpubox_filenames == NULL) {
        snprintf(errorMessage, MWALIB_ERROR_MESSAGE_LEN, "malloc failed for obs->gpubox_filenames");
        return EXIT_FAILURE;
    }
    obs->gpubox_ptrs = (fitsfile **)malloc(sizeof(fitsfile *) * obs->gpubox_filename_count);
    if (obs->gpubox_ptrs == NULL) {
        snprintf(errorMessage, MWALIB_ERROR_MESSAGE_LEN, "malloc failed for obs->gpubox_ptrs");
        return EXIT_FAILURE;
    }

    for (int i = 0; i < obs->gpubox_filename_count; i++) {
        obs->gpubox_filenames[i] = (char *)malloc(sizeof(char) * (strlen(args->gpubox_filenames[i]) + 1));
        if (obs->gpubox_filenames[i] == NULL) {
            snprintf(errorMessage, MWALIB_ERROR_MESSAGE_LEN, "malloc failed for obs->gpubox_filenames[%d]", i);
            return EXIT_FAILURE;
        }

        strcpy(obs->gpubox_filenames[i], args->gpubox_filenames[i]);

        if (open_fits(&obs->gpubox_ptrs[i], obs->gpubox_filenames[i], errorMessage) != EXIT_SUCCESS) {
            return EXIT_FAILURE;
        }
    }

    // Populate the gpubox batches.
    if (determine_gpubox_batches(obs, errorMessage) != EXIT_SUCCESS) {
        sprintf(errorMessage + strlen(errorMessage), " (determine_gpubox_batches)");
        return EXIT_FAILURE;
    }

    // CHANNELS
    char *coarse_channel_string = (char *)malloc(sizeof(char) * 1024);
    obs->coarse_channels = (int *)malloc(sizeof(int) * MWALIB_MAX_COARSE_CHANNELS);
    if (coarse_channel_string == NULL) {
        snprintf(errorMessage, MWALIB_ERROR_MESSAGE_LEN, "malloc failed for coarse_channel_string");
        return EXIT_FAILURE;
    }
    if (obs->coarse_channels == NULL) {
        snprintf(errorMessage, MWALIB_ERROR_MESSAGE_LEN, "malloc failed for obs->coarse_channels");
        return EXIT_FAILURE;
    }

    if (get_fits_comma_delimited_ints(obs->metafits_ptr, "CHANNELS", 1023, coarse_channel_string,
                                      &obs->num_coarse_channels, obs->coarse_channels, errorMessage) != EXIT_SUCCESS) {
        return EXIT_FAILURE;
    }
    free(coarse_channel_string);

    // Check that the number of coarse-band channels is also the same as the
    // number of files in a gpubox file batch.
    // TODO: Relax this constraint. Use a warning string for this purpose?
    if (obs->gpubox_filename_count / obs->gpubox_batch_count != obs->num_coarse_channels) {
        snprintf(
            errorMessage, MWALIB_ERROR_MESSAGE_LEN,
            "The number of gpubox files does not match the number of coarse-band channels specified by the metafits!");
        return EXIT_FAILURE;
    }

    // Populate the fine channels. For some reason, this isn't in the metafits.
    if (determine_gpubox_fine_channels(obs, errorMessage) != EXIT_SUCCESS) {
        sprintf(errorMessage + strlen(errorMessage), " (determine_gpubox_fine_channels)");
        return EXIT_FAILURE;
    }

    // Populate the start and end times of the observation.
    if (determine_obs_times(obs, errorMessage) != EXIT_SUCCESS) {
        sprintf(errorMessage + strlen(errorMessage), " (determine_obs_times)");
        return EXIT_FAILURE;
    }

    return EXIT_SUCCESS;
}
