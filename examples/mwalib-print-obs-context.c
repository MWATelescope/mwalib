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

    // Create metaafits context
    MetafitsContext *metafits_context = NULL;
    if (mwalib_metafits_context_new(argv[1],&metafits_context, error_message, ERROR_MESSAGE_LEN) != EXIT_SUCCESS)
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

    if (mwalib_correlator_context_display(correlator_context, error_message, ERROR_MESSAGE_LEN) != EXIT_SUCCESS)
    {
        printf("Error displaying context info: %s\n", error_message);
        exit(-1);
    }

    // Example of using metadata struct    
    mwalibMetafitsMetadata *metafits_metadata = NULL;
    if (mwalib_metafits_metadata_get(NULL, correlator_context, &metafits_metadata, error_message, ERROR_MESSAGE_LEN) == EXIT_SUCCESS)
    {
        printf("\nOutputting metafits metadata\n");
        printf("===============================================================================\n");
        printf("obsid:                          %d\n", metafits_metadata->obsid);
        printf("mwa_latitude:                   %f rad\n", metafits_metadata->mwa_latitude_radians);
        printf("mwa_longitude:                  %f rad\n", metafits_metadata->mwa_longitude_radians);
        printf("mwa_altitude:                   %f m\n", metafits_metadata->mwa_altitude_metres);
        printf("coax_v_factor:                  %f\n", metafits_metadata->coax_v_factor);
        printf("R.A. (tile pointing):           %f degrees\n", metafits_metadata->ra_tile_pointing_degrees);
        printf("Dec. (tile pointing):           %f degrees\n", metafits_metadata->dec_tile_pointing_degrees);
        printf("R.A. (phase centre):            %f degrees\n", metafits_metadata->ra_phase_center_degrees);
        printf("Dec. (phase centre):            %f degrees\n", metafits_metadata->dec_phase_center_degrees);
        printf("Azimuth:                        %f degrees\n", metafits_metadata->azimuth_degrees);
        printf("Altitude:                       %f degrees\n", metafits_metadata->altitude_degrees);
        printf("Sun altitude:                   %f degrees\n", metafits_metadata->sun_altitude_degrees);
        printf("Sun distance:                   %f degrees\n", metafits_metadata->sun_distance_degrees);
        printf("Moon distance:                  %f degrees\n", metafits_metadata->moon_distance_degrees);
        printf("Jupiter distance:               %f degrees\n", metafits_metadata->jupiter_distance_degrees);
        printf("LST:                            %f degrees\n", metafits_metadata->lst_degrees);
        printf("HA:                             %s H:M:S\n", metafits_metadata->hour_angle_string);
        printf("Grid name:                      %s\n", metafits_metadata->grid_name);
        printf("Grid number:                    %d\n", metafits_metadata->grid_number);
        printf("Creator:                        %s\n", metafits_metadata->creator);
        printf("Project Id:                     %s\n", metafits_metadata->project_id);
        printf("Observation Name:               %s\n", metafits_metadata->observation_name);
        printf("Mode:                           %s\n", metafits_metadata->mode);
        printf("Global analogue attenuation:    %f dB\n", metafits_metadata->global_analogue_attenuation_db);
        printf("Scheduled start:                %ld UNIX timestamp\n", metafits_metadata->scheduled_start_utc);
        printf("Scheduled start:                %f MJD\n", metafits_metadata->scheduled_start_mjd);
        printf("Scheduled duration:             %ld ms\n", metafits_metadata->scheduled_duration_milliseconds);
        printf("Quacktime:                      %ld ms\n", metafits_metadata->quack_time_duration_milliseconds);
        printf("Good UNIX time:                 %ld ms\n", metafits_metadata->good_time_unix_milliseconds);
        printf("num_antennas:                   %ld\n", metafits_metadata->num_antennas);
        printf("num_antenna_pols:               %ld\n", metafits_metadata->num_antenna_pols);
        printf("observation_bandwidth_hz:       %d\n", metafits_metadata->observation_bandwidth_hz);
        printf("coarse_channel_width_hz:        %d\n", metafits_metadata->coarse_channel_width_hz);
        printf("num_coarse_channels:            %ld\n", metafits_metadata->num_coarse_channels);
    }
    else
    {
        printf("Error %s", error_message);
        exit(1);
    }
    

    // Example of using timestep struct
    for (long x=0; x<1; x++)
    {
        mwalibTimeStep *ts = NULL;  // Should return all timesteps
        if (mwalib_correlator_timesteps_get(correlator_context, &ts, error_message, ERROR_MESSAGE_LEN) == EXIT_SUCCESS)
        {
            for (int i=0; i < corr_metadata->num_timesteps; i++)
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
    }    

    // Example of using coarse channels
    mwalibCoarseChannel *cc0 = NULL;
    if (mwalib_correlator_coarse_channel_get(correlator_context, 0, &cc0, error_message, ERROR_MESSAGE_LEN) == EXIT_SUCCESS)
    {
        printf("Coarse Channel 0 is %.2f MHz\n", (float)cc0->channel_centre_hz / 1000000.);
    }
    else
    {
        printf("Error getting Coarse Channel 0: %s\n", error_message);
    }

    // Example of using antennas  
    for (long x=0; x<999999; x++)
        {      
        mwalibAntenna *ants = NULL;
        if (mwalib_antennas_get(metafits_context, NULL, &ants, error_message, ERROR_MESSAGE_LEN) == EXIT_SUCCESS)
        {
            for (int i=0; i < metafits_metadata->num_antennas; i++)
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
    }

    // Example of using rf_inputs
    mwalibRFInput *rf0 = NULL;
    if (mwalib_rfinput_get(metafits_context, NULL, 0, &rf0, error_message, ERROR_MESSAGE_LEN) == EXIT_SUCCESS)
    {
        printf("rf_input 0 is %s %s\n", rf0->tile_name, rf0->pol);
    }
    else
    {
        printf("Error getting rf_input 0: %s\n", error_message);
    }

    // Example of using visibility pols
    // Below is to test for memory leaks
    for (long x=0; x<1; x++)
    {
        mwalibVisibilityPol *vis_pol_array = NULL;
        if (mwalib_correlator_visibility_pols_get(correlator_context, &vis_pol_array, error_message, ERROR_MESSAGE_LEN) == EXIT_SUCCESS)
        {
            for (int i=0; i < corr_metadata->num_visibility_pols; i++)
            {
                printf("Loop: %ld  mwalibVisibilityPols %d is %s\n", x, i, vis_pol_array[i].polarisation);
            }
        }
        else
        {
            printf("Error getting mwalibVisibilityPols %s\n", error_message);
            exit(-1);
        }
        // Clean up visibility pols
        mwalib_visibility_pols_free(vis_pol_array, corr_metadata->num_visibility_pols);
    }

    // Clean up coarse rf_inputs
    //mwalib_rfinput_free(rf0);

    // Clean up coarse channels
    //mwalib_coarse_channel_free(cc0);    

    // Clean up
    mwalib_correlator_metadata_free(corr_metadata);
    mwalib_correlator_context_free(correlator_context);

    free(gpuboxes);
    free(error_message);

    return EXIT_SUCCESS;
}
