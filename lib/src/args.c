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
    args->metafits_ptr = NULL;

    args->obs_id = 0;

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
    args->gpubox_filename_count = args->gpubox_filename_count + 1;

    args->gpubox_filenames[args->gpubox_filename_count] = (char *)malloc(sizeof(char) * (strlen(filename) + 1));
    strcpy(args->gpubox_filenames[args->gpubox_filename_count], filename);
    return EXIT_SUCCESS;
}

/**
 *
 *  @brief This function validates command line arguments. Returns success if
 * all good.
 *  @param[in] args Pointer to the mwalibArgs_s structure where we put the
 * parsed arguments.
 *  @param[inout] errorMessage Pointer to a string of length
 * ARG_ERROR_MESSAGE_LEN containing an error message or empty string if no error
 *  @returns EXIT_SUCCESS on success, or -1 if there was an error.
 */
int process_args(mwalibArgs_s *args, char *errorMessage)
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
    if (open_fits(&(args->metafits_ptr), args->metafits_filename, errorMessage) != EXIT_SUCCESS) {
        return EXIT_FAILURE;
    }

    // Get the OBSID
    if (get_fits_int_value(args->metafits_ptr, "GPSTIME", &(args->obs_id), errorMessage) != EXIT_SUCCESS) {
        return EXIT_FAILURE;
    }

    // CHANNELS
    char coarse_channel_string[1024] = {""};
    if (get_fits_string_value(args->metafits_ptr, "CHANNELS", coarse_channel_string, errorMessage) != EXIT_SUCCESS) {
        return EXIT_FAILURE;
    }
    printf("Coarse channels = %s\n", coarse_channel_string);

    return EXIT_SUCCESS;
}
