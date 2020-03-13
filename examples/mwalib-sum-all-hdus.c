// Given gpubox files, add their entire contents and report the sum.

#include <stdio.h>
#include <stdlib.h>

#include "fitsio.h"
#include "mwalib.h"

#define ERROR_MESSAGE_LEN 1024

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

    const char **gpuboxes = malloc(sizeof(char *) * (argc - 2));
    for (int i = 0; i < argc - 2; i++)
    {
        gpuboxes[i] = argv[i + 2];
    }

    // Allocate buffer for any error messages
    char *error_message = malloc(ERROR_MESSAGE_LEN * sizeof(char));

    mwalibContext *context = mwalibContext_get(argv[1], gpuboxes, argc - 2, error_message, ERROR_MESSAGE_LEN);

    mwalibMetadata *metadata = mwalibMetadata_get(context, error_message, ERROR_MESSAGE_LEN);

    int num_timesteps = metadata->num_timesteps;
    int num_coarse_channels = metadata->num_coarse_channels;
    int num_vis_pols = metadata->num_visibility_pols;
    int num_fine_channels = metadata->num_fine_channels_per_coarse;
    int num_baselines = metadata->num_baselines;
    long num_floats = metadata->num_timestep_coarse_channel_floats;
    float *data_buffer = calloc(num_floats * num_timesteps, sizeof(float));
    float *buffer_ptr = data_buffer; // Keep data_buffer pointing to the start of the buffer so we can free it later
    double sum = 0;

    for (int timestep_index = 0; timestep_index < num_timesteps; timestep_index++)
    {
        double this_sum = 0;

        for (int coarse_channel_index = 0; coarse_channel_index < num_coarse_channels; coarse_channel_index++)
        {
            if (mwalibContext_read_by_baseline(context, timestep_index, coarse_channel_index,
                                               buffer_ptr, num_floats, error_message, ERROR_MESSAGE_LEN) == EXIT_SUCCESS)
            {
                for (long i = 0; i < num_floats; i++)
                {
                    this_sum += buffer_ptr[i];
                }
            }
        }
        printf("Timestep: %d, Sum: %f\n", timestep_index, this_sum);

        // add to total sum
        sum += this_sum;

        // move destination buffer pointer along by the number of floats
        if (timestep_index < num_timesteps - 1)
        {
            buffer_ptr += num_floats;
        }
    }

    printf("Total sum: %f\n", sum);

    mwalibMetadata_free(metadata);
    mwalibContext_free(context);
    free(gpuboxes);
    free(error_message);
    free(data_buffer);

    return EXIT_SUCCESS;
}