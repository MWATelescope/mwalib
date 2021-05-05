// Given voltage files and a metafits file, provide metadata about this VCS observation
#include <stdio.h>
#include <stdlib.h>

#include "fitsio.h"
#include "mwalib.h"

#define ERROR_MESSAGE_LEN 1024

void do_sum(VoltageContext *context, long bytes_per_timestep, size_t num_timesteps, size_t num_coarse_chans)
{
    // Allocate buffer for any error messages
    char *error_message = malloc(ERROR_MESSAGE_LEN * sizeof(char));

    u_char *data_buffer = calloc(bytes_per_timestep * num_timesteps * num_coarse_chans, sizeof(float));
    u_char *buffer_ptr = data_buffer; // Keep data_buffer pointing to the start of the buffer so we can free it later
    double sum = 0;

    for (int timestep_index = 0; timestep_index < num_timesteps; timestep_index++)
    {
        double ts_sum = 0;

        for (int coarse_chan_index = 0; coarse_chan_index < num_coarse_chans; coarse_chan_index++)
        {
            double ts_cc_sum = 0;

            printf("Reading timestep: %d, Coarse Channel: %d...\n", timestep_index, coarse_chan_index);
            if (mwalib_voltage_context_read_file(context, timestep_index, coarse_chan_index, buffer_ptr, bytes_per_timestep, error_message, ERROR_MESSAGE_LEN) == EXIT_SUCCESS)
            {
                printf("Summing...");
                for (long i = 0; i < bytes_per_timestep; i++)
                {
                    ts_cc_sum += buffer_ptr[i];
                }

                printf("sum: %f.\n", ts_cc_sum);
            }

            ts_sum += ts_cc_sum;
        }

        // add to total sum
        sum += ts_sum;

        // move destination buffer pointer along by the number of floats
        if (timestep_index < num_timesteps - 1)
        {
            buffer_ptr += bytes_per_timestep;
        }
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

    const char **voltage_files = malloc(sizeof(char *) * (argc - 2));
    for (int i = 0; i < argc - 2; i++)
    {
        voltage_files[i] = argv[i + 2];
    }

    // Allocate buffer for any error messages
    char *error_message = malloc(ERROR_MESSAGE_LEN * sizeof(char));

    VoltageContext *volt_context = NULL;

    if (mwalib_voltage_context_new(argv[1], voltage_files, argc - 2, &volt_context, error_message, ERROR_MESSAGE_LEN) != EXIT_SUCCESS)
    {
        printf("Error creating correlator context: %s\n", error_message);
        exit(-1);
    }

    // Print summary of correlator context
    if (mwalib_voltage_context_display(volt_context, error_message, ERROR_MESSAGE_LEN) != EXIT_SUCCESS)
    {
        printf("Error displaying correlator context summary: %s\n", error_message);
        exit(-1);
    }

    VoltageMetadata *volt_metadata = NULL;

    if (mwalib_voltage_metadata_get(volt_context, &volt_metadata, error_message, ERROR_MESSAGE_LEN) != EXIT_SUCCESS)
    {
        printf("Error getting correlator metadata: %s\n", error_message);
        exit(-1);
    }

    MetafitsMetadata *metafits_metadata = NULL;

    if (mwalib_metafits_metadata_get(NULL, NULL, volt_context, &metafits_metadata, error_message, ERROR_MESSAGE_LEN) != EXIT_SUCCESS)
    {
        printf("Error getting metafits metadata: %s\n", error_message);
        exit(-1);
    }

    int num_timesteps = volt_metadata->num_timesteps;
    int num_coarse_chans = volt_metadata->num_coarse_chans;
    long num_bytes_per_timestep = volt_metadata->num_voltage_blocks_per_timestep * volt_metadata->voltage_block_size_bytes;

    // Now sum the data
    do_sum(volt_context, num_bytes_per_timestep, num_timesteps, num_coarse_chans);

    mwalib_metafits_metadata_free(metafits_metadata);
    mwalib_voltage_metadata_free(volt_metadata);
    mwalib_voltage_context_free(volt_context);
    free(voltage_files);
    free(error_message);

    return EXIT_SUCCESS;
}