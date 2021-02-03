/*!
Example code to sum all the hdu's given a metafits and one or more gpubox files
*/

#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
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

    // Create correlator context
    CorrelatorContext *correlator_context;
    if (mwalib_correlator_context_new(argv[1], gpuboxes, argc - 2, &correlator_context, error_message, ERROR_MESSAGE_LEN) != EXIT_SUCCESS)
    {
        printf("Error getting correlator context: %s\n", error_message);
        exit(-1);
    }

    // Create metafits context
    MetafitsContext *metafits_context = NULL;
    if (mwalib_metafits_context_new(argv[1], &metafits_context, error_message, ERROR_MESSAGE_LEN) != EXIT_SUCCESS)
    {
        printf("Error getting metaafits context: %s\n", error_message);
        exit(-1);
    }

    // Get correlator metadata
    mwalibCorrelatorMetadata *corr_metadata = NULL;
    if (mwalib_correlator_metadata_get(correlator_context, &corr_metadata, error_message, ERROR_MESSAGE_LEN) != EXIT_SUCCESS)
    {
        printf("Error displaying correlator metadata info: %s\n", error_message);
        exit(-1);
    }

    // Example of using metadata struct
    mwalibMetafitsMetadata *metafits_metadata = NULL;
    if (mwalib_metafits_metadata_get(NULL, correlator_context, &metafits_metadata, error_message, ERROR_MESSAGE_LEN) != EXIT_SUCCESS)
    {
        printf("Error %s", error_message);
        exit(1);
    }

    printf("Retrieved metadata for obsid: %d\n", metafits_metadata->obsid);

    if (mwalib_correlator_context_display(correlator_context, error_message, ERROR_MESSAGE_LEN) != EXIT_SUCCESS)
    {
        printf("Error displaying context info: %s\n", error_message);
        exit(-1);
    }

    // Example of using antennas
    mwalibAntenna *ants = NULL;
    size_t nants = 0;

    if (mwalib_antennas_get(metafits_context, NULL, &ants, &nants, error_message, ERROR_MESSAGE_LEN) == EXIT_SUCCESS)
    {
        printf("%lu antennas returned\n", nants);

        for (int i = 0; i < metafits_metadata->num_antennas; i++)
        {
            printf("antenna %d is %s\n", i, ants[i].tile_name);
        }
    }
    else
    {
        printf("Error getting antennas: %s\n", error_message);
    }

    // Clean up antennas
    mwalib_antennas_free(ants, metafits_metadata->num_antennas);

    // Example of using baselines
    mwalibBaseline *bls = NULL;
    size_t nbls = 0;

    if (mwalib_correlator_baselines_get(correlator_context, &bls, &nbls, error_message, ERROR_MESSAGE_LEN) == EXIT_SUCCESS)
    {
        printf("%lu baselines returned\n", nbls);

        for (int i = 0; i < corr_metadata->num_baselines; i++)
        {
            printf("Baseline %d is ant %lu vs ant %lu\n", i, bls[i].antenna1_index, bls[i].antenna2_index);
        }
    }
    else
    {
        printf("Error getting baselines: %s\n", error_message);
    }

    // Clean up baselines
    mwalib_baselines_free(bls, corr_metadata->num_baselines);

    // Example of using coarse channels
    mwalibCoarseChannel *ccs = NULL;
    size_t nccs = 0;

    if (mwalib_correlator_coarse_channels_get(correlator_context, &ccs, &nccs, error_message, ERROR_MESSAGE_LEN) == EXIT_SUCCESS)
    {
        printf("%lu Coarse Channels returned\n", nccs);

        for (int i = 0; i < corr_metadata->num_coarse_channels; i++)
        {
            printf("Coarse Channel %d is %.2f MHz\n", i, (float)ccs[i].channel_centre_hz / 1000000.);
        }
    }
    else
    {
        printf("Error getting Coarse Channels: %s\n", error_message);
    }

    // Clean up coarse channels
    mwalib_coarse_channels_free(ccs, corr_metadata->num_coarse_channels);

    // Example of using rf_inputs
    mwalibRFInput *rfs = NULL;
    size_t nrfs = 0;

    if (mwalib_rfinputs_get(NULL, correlator_context, &rfs, &nrfs, error_message, ERROR_MESSAGE_LEN) == EXIT_SUCCESS)
    {
        printf("%lu rf_inputs returned\n", nrfs);

        for (int i = 0; i < metafits_metadata->num_rf_inputs; i++)
        {
            printf("rf_input %d is %s %s\n", i, rfs[i].tile_name, rfs[i].pol);
        }
    }
    else
    {
        printf("Error getting rf_inputs: %s\n", error_message);
    }

    // Clean up rf_inputs
    mwalib_rfinputs_free(rfs, metafits_metadata->num_rf_inputs);

    // Example of using timestep struct
    mwalibTimeStep *ts = NULL;
    size_t nts = 0;

    if (mwalib_correlator_timesteps_get(correlator_context, &ts, &nts, error_message, ERROR_MESSAGE_LEN) == EXIT_SUCCESS)
    {
        printf("%lu timesteps returned\n", nts);

        for (int i = 0; i < corr_metadata->num_timesteps; i++)
        {
            printf("Timestep %d is %.2f\n", i, ts[i].unix_time_ms / 1000.);
        }
    }
    else
    {
        printf("Error getting timesteps: %s\n", error_message);
    }

    // Clean up timesteps
    mwalib_timesteps_free(ts, corr_metadata->num_timesteps);

    // Example of using visibility pols
    mwalibVisibilityPol *vis_pol_array = NULL;
    size_t nvps = 0;

    if (mwalib_correlator_visibility_pols_get(correlator_context, &vis_pol_array, &nvps, error_message, ERROR_MESSAGE_LEN) == EXIT_SUCCESS)
    {
        printf("%lu visibility pols returned\n", nvps);

        for (int i = 0; i < corr_metadata->num_visibility_pols; i++)
        {
            printf("mwalibVisibilityPols %d is %s\n", i, vis_pol_array[i].polarisation);
        }
    }
    else
    {
        printf("Error getting mwalibVisibilityPols: %s\n", error_message);
        exit(-1);
    }
    // Clean up visibility pols
    mwalib_visibility_pols_free(vis_pol_array, corr_metadata->num_visibility_pols);

    // Clean up
    mwalib_correlator_metadata_free(corr_metadata);
    mwalib_correlator_context_free(correlator_context);

    free(gpuboxes);
    free(error_message);

    return EXIT_SUCCESS;
}
