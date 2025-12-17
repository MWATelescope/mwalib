// Given voltage files and a metafits file, provide metadata about this VCS observation
#include <stdio.h>
#include <stdlib.h>

#include "fitsio.h"
#include "mwalib.h"

#define ERROR_MESSAGE_LEN 1024
#define DISPLAY_MESSAGE_LEN 32768

void do_sum(VoltageContext *context, long bytes_per_timestep, size_t num_timesteps, size_t num_coarse_chans, size_t num_provided_timesteps, size_t num_provided_coarse_chans)
{
    // Allocate buffer for any error messages
    char *error_message = malloc(ERROR_MESSAGE_LEN * sizeof(char));

    signed char *data_buffer = calloc(bytes_per_timestep * num_provided_timesteps * num_provided_coarse_chans, sizeof(signed char));
    signed char *buffer_ptr = data_buffer; // Keep data_buffer pointing to the start of the buffer so we can free it later
    double sum = 0;

    for (size_t timestep_index = 0; timestep_index < num_timesteps; timestep_index++)
    {
        double ts_sum = 0;

        for (size_t coarse_chan_index = 0; coarse_chan_index < num_coarse_chans; coarse_chan_index++)
        {
            double ts_cc_sum = 0;

            if (mwalib_voltage_context_read_file(context, timestep_index, coarse_chan_index, buffer_ptr, bytes_per_timestep, error_message, ERROR_MESSAGE_LEN) == EXIT_SUCCESS)
            {
                printf("Reading data from timestep: %ld, Coarse Channel: %ld...\n", timestep_index, coarse_chan_index);
                for (long i = 0; i < bytes_per_timestep; i++)
                {
                    ts_cc_sum += buffer_ptr[i];
                }

                printf("sum: %f.\n", ts_cc_sum);

                // Move pointer along
                buffer_ptr += bytes_per_timestep;
            }

            ts_sum += ts_cc_sum;
        }

        // add to total sum
        sum += ts_sum;
    }

    printf("Total sum: %f\n", sum);
    free(data_buffer);
    free(error_message);
}

int main(int argc, char *argv[])
{
    // Assume that the first file provided is the metafits file, and all others
    // are voltage data files. Therefore, we need at least two files provided to main,
    // such that there's at least one voltage data file.
    if (argc < 3)
    {
        printf("At least two files are needed.\n");
        return EXIT_FAILURE;
    }

    // Allocate buffer for any error messages
    char *error_message = malloc(ERROR_MESSAGE_LEN * sizeof(char));

    VoltageContext *volt_context = NULL;

    if (mwalib_voltage_context_new(argv[1], (const char **)&argv[2], argc - 2, &volt_context, error_message, ERROR_MESSAGE_LEN) != EXIT_SUCCESS)
    {
        printf("Error creating voltage context: %s\n", error_message);
        free(error_message);
        exit(-1);
    }

    // Print version
    printf("Using mwalib v%d.%d.%d\n", mwalib_get_version_major(), mwalib_get_version_minor(), mwalib_get_version_patch());

    // Allocate buffer space for the display info
    char *display_message = malloc(DISPLAY_MESSAGE_LEN * sizeof(char));

    // Print summary of correlator context
    if (mwalib_voltage_context_display(volt_context, display_message, DISPLAY_MESSAGE_LEN, error_message, ERROR_MESSAGE_LEN) != EXIT_SUCCESS)
    {
        printf("Error displaying voltage context summary: %s\n", error_message);
        free(error_message);
        exit(-1);
    }
    else
    {
        printf("%s\n", display_message);
        free(display_message);
    }

    VoltageMetadata *volt_metadata = NULL;

    if (mwalib_voltage_metadata_get(volt_context, &volt_metadata, error_message, ERROR_MESSAGE_LEN) != EXIT_SUCCESS)
    {
        printf("Error getting voltage metadata: %s\n", error_message);
        free(error_message);
        exit(-1);
    }

    MetafitsMetadata *metafits_metadata = NULL;

    if (mwalib_metafits_metadata_get(NULL, NULL, volt_context, &metafits_metadata, error_message, ERROR_MESSAGE_LEN) != EXIT_SUCCESS)
    {
        printf("Error getting metafits metadata: %s\n", error_message);
        free(error_message);
        exit(-1);
    }

    int num_timesteps = volt_metadata->num_timesteps;
    int num_coarse_chans = volt_metadata->num_coarse_chans;
    int num_provided_timesteps = volt_metadata->num_provided_timesteps;
    int num_provided_coarse_chans = volt_metadata->num_provided_coarse_chans;
    long num_bytes_per_timestep = volt_metadata->num_voltage_blocks_per_timestep * volt_metadata->voltage_block_size_bytes;

    // Now sum the data
    do_sum(volt_context, num_bytes_per_timestep, num_timesteps, num_coarse_chans, num_provided_timesteps, num_provided_coarse_chans);

    mwalib_metafits_metadata_free(metafits_metadata);
    mwalib_voltage_metadata_free(volt_metadata);
    mwalib_voltage_context_free(volt_context);
    free(error_message);

    return EXIT_SUCCESS;
}