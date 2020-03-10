#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

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
    mwalibContext_display(context);

    // Test metadata struct
    mwalibMetadata *metadata = mwalibMetadata_get(context);
    printf("\nOutputting metadata\n");
    printf("===============================================================================\n");
    printf("obsid:                          %d\n", metadata->obsid);

    switch (metadata->corr_version)
    {
    case V2:
        printf("correlator version:             V2\n");
        break;
    case Legacy:
        printf("correlator version:             Legacy\n");
        break;
    case OldLegacy:
        printf("correlator version:             Old Legacy\n");
        break;
    default:
        printf("correlator version:             Unknown\n");
        break;
    }

    printf("coax_v_factor:                  %f\n", metadata->coax_v_factor);
    printf("start_unix_time_milliseconds:   %ld\n", metadata->start_unix_time_milliseconds);
    printf("end_unix_time_milliseconds:     %ld\n", metadata->end_unix_time_milliseconds);
    printf("duration_milliseconds:          %ld\n", metadata->duration_milliseconds);
    printf("integration_time_milliseconds:  %ld\n", metadata->integration_time_milliseconds);
    printf("num_timesteps:                  %ld\n", metadata->num_timesteps);
    printf("num_antennas:                   %ld\n", metadata->num_antennas);
    printf("num_baselines:                  %ld\n", metadata->num_baselines);
    printf("num_antenna_pols:               %ld\n", metadata->num_antenna_pols);
    printf("num_visibility_pols:            %ld\n", metadata->num_visibility_pols);
    printf("observation_bandwidth_hz:       %d\n", metadata->observation_bandwidth_hz);
    printf("coarse_channel_width_hz:        %d\n", metadata->coarse_channel_width_hz);
    printf("num_coarse_channels:            %ld\n", metadata->num_coarse_channels);
    printf("num_fine_channels_per_coarse:   %ld\n", metadata->num_fine_channels_per_coarse);
    printf("fine_channel_width_hz:          %d\n", metadata->fine_channel_width_hz);
    printf("num_gpubox_files:               %ld\n", metadata->num_gpubox_files);
    printf("timestep_coarse_channel_floats: %ld\n", metadata->timestep_coarse_channel_floats);
    printf("timestep_coarse_channel_bytes:  %ld\n", metadata->timestep_coarse_channel_bytes);

    // Test timestep struct
    mwalibTimeStep *ts0 = mwalibTimeStep_get(context, 0); // Should return first timestep
    if (ts0 != NULL)
    {
        printf("Timestep 0 is %lu\n", ts0->unix_time_ms / 1000);
    }
    else
    {
        printf("Error getting timestep 0\n");
    }

    mwalibTimeStep *ts90 = mwalibTimeStep_get(context, 90); // Should return NULL
    if (ts90 != NULL)
    {
        printf("Timestep 90 is %lu\n", ts90->unix_time_ms / 1000);
    }
    else
    {
        printf("Error getting timestep 90\n");
    }

    // Clean up timesteps
    mwalibTimeStep_free(ts0);
    mwalibTimeStep_free(ts90);

    // Clean up
    mwalibMetadata_free(metadata);
    mwalibContext_free(context);

    free(gpuboxes);

    return EXIT_SUCCESS;
}
