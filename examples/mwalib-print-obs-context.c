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

    mwalibContext *context = mwalibContext_get(argv[1], gpuboxes, argc - 2, error_message, ERROR_MESSAGE_LEN);

    if (mwalibContext_display(context, error_message, ERROR_MESSAGE_LEN) != EXIT_SUCCESS)
    {
        printf("Error displaying context info: %s\n", error_message);
        exit(-1);
    }

    // Example of using metadata struct
    mwalibMetadata *metadata = mwalibMetadata_get(context, error_message, ERROR_MESSAGE_LEN);
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

    printf("mwa_latitude:                   %f rad\n", metadata->mwa_latitude_radians);
    printf("mwa_longitude:                  %f rad\n", metadata->mwa_longitude_radians);
    printf("mwa_altitude:                   %f m\n", metadata->mwa_altitude_metres);

    printf("coax_v_factor:                  %f\n", metadata->coax_v_factor);

    printf("R.A. (tile pointing):           %f degrees\n", metadata->ra_tile_pointing_degrees);
    printf("Dec. (tile pointing):           %f degrees\n", metadata->dec_tile_pointing_degrees);
    printf("R.A. (phase centre):            %f degrees\n", metadata->ra_phase_center_degrees);
    printf("Dec. (phase centre):            %f degrees\n", metadata->dec_phase_center_degrees);
    printf("Azimuth:                        %f degrees\n", metadata->azimuth_degrees);
    printf("Altitude:                       %f degrees\n", metadata->altitude_degrees);
    printf("Sun altitude:                   %f degrees\n", metadata->sun_altitude_degrees);
    printf("Sun distance:                   %f degrees\n", metadata->sun_distance_degrees);
    printf("Moon distance:                  %f degrees\n", metadata->moon_distance_degrees);
    printf("Jupiter distance:               %f degrees\n", metadata->jupiter_distance_degrees);
    printf("LST:                            %f degrees\n", metadata->lst_degrees);
    printf("HA:                             %s H:M:S\n", metadata->hour_angle_string);
    printf("Grid name:                      %s\n", metadata->grid_name);
    printf("Grid number:                    %d\n", metadata->grid_number);
    printf("Creator:                        %s\n", metadata->creator);
    printf("Project Id:                     %s\n", metadata->project_id);
    printf("Observation Name:               %s\n", metadata->observation_name);
    printf("Mode:                           %s\n", metadata->mode);
    printf("Global analogue attenuation:    %f dB\n", metadata->global_analogue_attenuation_db);
    printf("Scheduled start:                %ld UNIX timestamp\n", metadata->scheduled_start_utc);
    printf("Scheduled start:                %f MJD\n", metadata->scheduled_start_mjd);
    printf("Scheduled duration:             %ld ms\n", metadata->scheduled_duration_milliseconds);

    printf("Start UNIX time:                %ld ms\n", metadata->start_unix_time_milliseconds);
    printf("End UNIX time:                  %ld ms\n", metadata->end_unix_time_milliseconds);
    printf("Duration:                       %ld ms\n", metadata->duration_milliseconds);

    printf("Quacktime:                      %ld ms\n", metadata->quack_time_duration_milliseconds);
    printf("Good UNIX time:                 %ld ms\n", metadata->good_time_unix_milliseconds);

    printf("integration_time:               %ld ms\n", metadata->integration_time_milliseconds);
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
    printf("timestep_coarse_channel_floats: %ld\n", metadata->num_timestep_coarse_channel_floats);
    printf("timestep_coarse_channel_bytes:  %ld\n", metadata->num_timestep_coarse_channel_bytes);

    // Example of using timestep struct
    mwalibTimeStep *ts0 = mwalibTimeStep_get(context, 0, error_message, ERROR_MESSAGE_LEN); // Should return first timestep
    if (ts0 != NULL)
    {
        printf("Timestep 0 is %lu\n", ts0->unix_time_ms / 1000);
    }
    else
    {
        printf("Error getting timestep 0: %s\n", error_message);
    }

    // Example of using coarse channels
    mwalibCoarseChannel *cc0 = mwalibCoarseChannel_get(context, 0, error_message, ERROR_MESSAGE_LEN);
    if (cc0 != NULL)
    {
        printf("Coarse Channel 0 is %.2f MHz\n", (float)cc0->channel_centre_hz / 1000000.);
    }
    else
    {
        printf("Error getting Coarse Channel 0: %s\n", error_message);
    }

    // Example of using antennas
    mwalibAntenna *ant0 = mwalibAntenna_get(context, 0, error_message, ERROR_MESSAGE_LEN);
    if (ant0 != NULL)
    {
        printf("antenna 0 is %s\n", ant0->tile_name);
    }
    else
    {
        printf("Error getting antenna 0: %s\n", error_message);
    }

    // Example of using rf_inputs
    mwalibRFInput *rf0 = mwalibRFInput_get(context, 0, error_message, ERROR_MESSAGE_LEN);
    if (rf0 != NULL)
    {
        printf("rf_input 0 is %s %s\n", rf0->tile_name, rf0->pol);
    }
    else
    {
        printf("Error getting rf_input 0: %s\n", error_message);
    }

    // Clean up coarse rf_inputs
    mwalibRFInput_free(rf0);

    // Clean up antennas
    mwalibAntenna_free(ant0);

    // Clean up coarse channels
    mwalibCoarseChannel_free(cc0);

    // Clean up timesteps
    mwalibTimeStep_free(ts0);

    // Clean up
    mwalibMetadata_free(metadata);
    mwalibContext_free(context);

    free(gpuboxes);
    free(error_message);

    return EXIT_SUCCESS;
}
