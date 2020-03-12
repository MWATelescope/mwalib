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

    float *data_buffer = NULL;

    int num_timesteps = metadata->num_timesteps;
    int num_coarse_channels = metadata->num_coarse_channels;
    int num_vis_pols = metadata->num_visibility_pols;
    int num_fine_channels = metadata->num_fine_channels_per_coarse;
    int num_baselines = metadata->num_baselines;
    long num_floats = num_baselines * num_fine_channels * num_vis_pols * 2;

    double sum = 0;

    for (int timestep_index = 0; timestep_index < num_timesteps; timestep_index++)
    {
        for (int coarse_channel_index = 0; coarse_channel_index < num_coarse_channels; coarse_channel_index++)
        {
            data_buffer = mwalibContext_read_by_baseline(context, &timestep_index, &coarse_channel_index, error_message, ERROR_MESSAGE_LEN);
            if (data_buffer != NULL)
            {
                for (long i = 0; i < num_floats; i++)
                {
                    sum += data_buffer[i];
                }
            }
            free(data_buffer);
        }
    }

    printf("Total sum: %f\n", sum);

    mwalibMetadata_free(metadata);
    mwalibContext_free(context);
    free(gpuboxes);
    free(error_message);

    return EXIT_SUCCESS;
}