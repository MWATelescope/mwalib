// Given gpubox files, add their entire contents and report the sum.

#include <stdio.h>
#include <stdlib.h>

#include "fitsio.h"
#include "mwalib.h"

#define ERROR_MESSAGE_LEN 1024
#define DISPLAY_MESSAGE_LEN 32768

void do_sum(int mode, CorrelatorContext *context, long num_floats, int num_timesteps, int num_coarse_chans)
{
    // Allocate buffer for any error messages
    char *error_message = malloc(ERROR_MESSAGE_LEN * sizeof(char));

    float *data_buffer = calloc(num_floats * num_timesteps, sizeof(float));
    float *buffer_ptr = data_buffer; // Keep data_buffer pointing to the start of the buffer so we can free it later
    double sum = 0;

    int32_t (*sum_func)(CorrelatorContext *, uintptr_t, uintptr_t, float *, size_t, char *, size_t);

    switch (mode)
    {
    case 1:
        sum_func = mwalib_correlator_context_read_by_baseline;
        break;
    case 2:
        sum_func = mwalib_correlator_context_read_by_frequency;
        break;
    default:
        return;
    }

    for (int timestep_index = 0; timestep_index < num_timesteps; timestep_index++)
    {
        double this_sum = 0;

        for (int coarse_chan_index = 0; coarse_chan_index < num_coarse_chans; coarse_chan_index++)
        {
            int read_result = sum_func(context, timestep_index, coarse_chan_index, buffer_ptr, num_floats, error_message, ERROR_MESSAGE_LEN);

            if (read_result == MWALIB_SUCCESS)
            {
                for (long i = 0; i < num_floats; i++)
                {
                    this_sum += buffer_ptr[i];
                }
            }
            else if (read_result == MWALIB_NO_DATA_FOR_TIMESTEP_COARSECHAN)
            {
                // Skip this timestep/cc combo
            }
            else
            {
                printf("Error reading MWA data %s:\n", error_message);
                free(data_buffer);
                free(error_message);
                return;
            }
        }

        // Include the below for timestep level debug
        // printf("Timestep: %d, Sum: %f\n", timestep_index, this_sum);

        // add to total sum
        sum += this_sum;

        // move destination buffer pointer along by the number of floats
        if (timestep_index < num_timesteps - 1)
        {
            buffer_ptr += num_floats;
        }
    }

    printf("Total sum using mode %d: %f\n", mode, sum);
    free(data_buffer);
    free(error_message);
}

int main(int argc, char *argv[])
{
    // Assume that the first file provided is the metafits file, and all others
    // are gpubox files. Therefore, we need at least two files provided to main,
    // such that there's at least one gpubox file.
    if (argc < 3)
    {
        printf("At least two files are needed.\n");
        return EXIT_FAILURE;
    }

    // Allocate buffer for any error messages
    char *error_message = malloc(ERROR_MESSAGE_LEN * sizeof(char));

    CorrelatorContext *corr_context = NULL;

    if (mwalib_correlator_context_new(argv[1], (const char **)&argv[2], argc - 2, &corr_context, error_message, ERROR_MESSAGE_LEN) != EXIT_SUCCESS)
    {
        printf("Error creating correlator context: %s\n", error_message);
        free(error_message);
        exit(EXIT_FAILURE);
    }

    // Print summary of correlator context
    // Allocate buffer space for the display info
    char *display_message = malloc(DISPLAY_MESSAGE_LEN * sizeof(char));

    if (mwalib_correlator_context_display(corr_context, display_message, DISPLAY_MESSAGE_LEN, error_message, ERROR_MESSAGE_LEN) != EXIT_SUCCESS)
    {
        printf("Error displaying correlator context summary: %s\n", error_message);
        free(error_message);
        free(display_message);
        exit(EXIT_FAILURE);
    }
    else
    {
        printf("%s\n", display_message);
        free(display_message);
    }

    CorrelatorMetadata *corr_metadata = NULL;

    if (mwalib_correlator_metadata_get(corr_context, &corr_metadata, error_message, ERROR_MESSAGE_LEN) != EXIT_SUCCESS)
    {
        printf("Error getting correlator metadata: %s\n", error_message);
        free(error_message);
        exit(EXIT_FAILURE);
    }

    MetafitsMetadata *metafits_metadata = NULL;

    if (mwalib_metafits_metadata_get(NULL, corr_context, NULL, &metafits_metadata, error_message, ERROR_MESSAGE_LEN) != EXIT_SUCCESS)
    {
        printf("Error getting metafits metadata: %s\n", error_message);
        free(error_message);
        exit(EXIT_FAILURE);
    }

    int num_timesteps = corr_metadata->num_timesteps;
    int num_coarse_chans = corr_metadata->num_coarse_chans;
    long num_floats = corr_metadata->num_timestep_coarse_chan_floats;

    // Now sum by baseline
    do_sum(1, corr_context, num_floats, num_timesteps, num_coarse_chans);

    // Now sum by freq
    do_sum(2, corr_context, num_floats, num_timesteps, num_coarse_chans);

    mwalib_metafits_metadata_free(metafits_metadata);
    mwalib_correlator_metadata_free(corr_metadata);
    mwalib_correlator_context_free(corr_context);
    free(error_message);

    return EXIT_SUCCESS;
}