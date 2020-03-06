// Given gpubox files, add their entire contents and report the sum.

#include <stdio.h>
#include <stdlib.h>

#include "fitsio.h"
#include "mwalib.h"

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

    mwalibContext *context = mwalibContext_new(argv[1], gpuboxes, argc - 2);

    float *data_buffer = NULL;

    int num_timesteps = 53;
    int num_coarse_channels = 1;
    int num_vis_pols = 4;
    int num_fine_channels = 128;
    long num_floats = num_fine_channels * num_fine_channels * num_vis_pols * 2;

    double sum = 0;

    for (int timestep_index = 0; timestep_index < num_timesteps; timestep_index++)
    {
        for (int coarse_channel_index = 0; coarse_channel_index < num_coarse_channels; coarse_channel_index++)
        {
            data_buffer = mwalibContext_read_one_timestep_coarse_channel_bfp(context, &timestep_index, &coarse_channel_index);
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

    mwalibContext_free(context);
    free(gpuboxes);

    return EXIT_SUCCESS;
}
