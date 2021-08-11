/*!
Example code to print context info, given at least a metafits file and optionally and one or more gpubox files or voltage data files
*/
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "mwalib.h"
#include "time.h"

#define ERROR_MESSAGE_LEN 1024

void print_usage()
{
    printf("print-obs-context metafits_file [data_files...]\n");
    exit(0);
}

int main(int argc, char *argv[])
{
    // Assume that the first file provided is the metafits file, and all others
    // are gpubox files or voltage files. Therefore, we need at least one file provided to main.
    int file_count = argc - 1;

    if (file_count < 1)
    {
        printf("At least one file is needed (if only one, it should be the metafits file).\n");
        print_usage();
    }

    // Allocate buffer for any error messages
    char *error_message = malloc(ERROR_MESSAGE_LEN * sizeof(char));

    // Create context pointers
    MetafitsContext *metafits_context = NULL;
    CorrelatorContext *correlator_context = NULL;
    VoltageContext *voltage_context = NULL;

    if (file_count == 1)
    {
        // Metafits only
        if (mwalib_metafits_context_new2(argv[1], &metafits_context, error_message, ERROR_MESSAGE_LEN) != EXIT_SUCCESS)
        {
            printf("Error getting metafits context: %s\n", error_message);
            exit(-1);
        }

        // print metafits context info
        if (mwalib_metafits_context_display(metafits_context, error_message, ERROR_MESSAGE_LEN) != EXIT_SUCCESS)
        {
            printf("Error displaying metafits context info: %s\n", error_message);
            exit(-1);
        }
    }
    else
    {
        // Determine file type from first data file
        if (strcmp(strrchr(argv[2], '\0') - 5, ".fits") == 0)
        {
            // Correlator files
            const char **files = malloc(sizeof(char *) * (argc - 2));
            for (int i = 0; i < argc - 2; i++)
            {
                files[i] = argv[i + 2];
            }

            // Create correlator context
            if (mwalib_correlator_context_new(argv[1], files, file_count - 1, &correlator_context, error_message, ERROR_MESSAGE_LEN) != EXIT_SUCCESS)
            {
                printf("Error getting correlator context: %s\n", error_message);
                exit(-1);
            }

            // Get correlator metadata
            CorrelatorMetadata *corr_metadata = NULL;
            if (mwalib_correlator_metadata_get(correlator_context, &corr_metadata, error_message, ERROR_MESSAGE_LEN) != EXIT_SUCCESS)
            {
                printf("Error retrieving correlator metadata info: %s\n", error_message);
                exit(-1);
            }

            // print correlator context info
            if (mwalib_correlator_context_display(correlator_context, error_message, ERROR_MESSAGE_LEN) != EXIT_SUCCESS)
            {
                printf("Error displaying context info: %s\n", error_message);
                exit(-1);
            }

            printf("\n\nExample of accessing Correlator Metadata:\n");
            if (corr_metadata->num_common_timesteps > 0)
            {
                printf("First common correlator timestep: is index %ld, and starts at %f Unix time\n", corr_metadata->common_timestep_indices[0], (double)corr_metadata->timesteps[corr_metadata->common_timestep_indices[0]].unix_time_ms / 1000.);
            }
            else
            {
                printf("No common timesteps\n");
            }

            if (corr_metadata->num_common_coarse_chans > 0)
            {
                printf("First common correlator coarse channel: is index %ld, and starts at %f MHz\n", corr_metadata->common_coarse_chan_indices[0], (float)corr_metadata->coarse_chans[corr_metadata->common_coarse_chan_indices[0]].chan_start_hz / 1000000.);
            }
            else
            {
                printf("No common coarse channels\n");
            }

            // Clean up metadata
            mwalib_correlator_metadata_free(corr_metadata);

            free(files);
        }
        else if (strcmp(strrchr(argv[2], '\0') - 4, ".sub") == 0 || strcmp(strrchr(argv[2], '\0') - 4, ".dat") == 0)
        {
            // Voltage files
            const char **files = malloc(sizeof(char *) * (argc - 2));
            for (int i = 0; i < argc - 2; i++)
            {
                files[i] = argv[i + 2];
            }

            // Create correlator context
            if (mwalib_voltage_context_new(argv[1], files, file_count - 1, &voltage_context, error_message, ERROR_MESSAGE_LEN) != EXIT_SUCCESS)
            {
                printf("Error getting correlator context: %s\n", error_message);
                exit(-1);
            }

            // Get voltage metadata
            VoltageMetadata *volt_metadata = NULL;
            if (mwalib_voltage_metadata_get(voltage_context, &volt_metadata, error_message, ERROR_MESSAGE_LEN) != EXIT_SUCCESS)
            {
                printf("Error retrieving voltage metadata info: %s\n", error_message);
                exit(-1);
            }

            // print voltage context info
            if (mwalib_voltage_context_display(voltage_context, error_message, ERROR_MESSAGE_LEN) != EXIT_SUCCESS)
            {
                printf("Error displaying voltage context info: %s\n", error_message);
                exit(-1);
            }

            printf("\n\nExample of accessing Voltage Metadata:\n");

            if (volt_metadata->num_common_timesteps > 0)
            {
                printf("First common voltage timestep: is index %ld, and starts at %f Unix time\n", volt_metadata->common_timestep_indices[0], (double)volt_metadata->timesteps[volt_metadata->common_timestep_indices[0]].unix_time_ms / 1000.);
            }
            else
            {
                printf("No common timesteps\n");
            }

            if (volt_metadata->num_common_coarse_chans > 0)
            {
                printf("First common voltage coarse channel: is index %ld, and starts at %f MHz\n", volt_metadata->common_coarse_chan_indices[0], (float)volt_metadata->coarse_chans[volt_metadata->common_coarse_chan_indices[0]].chan_start_hz / 1000000.);
            }
            else
            {
                printf("No common coarse channels\n");
            }

            // Clean up metadata
            mwalib_voltage_metadata_free(volt_metadata);

            free(files);
        }
        else
        {
            // Unknown files!
            printf("Error- provided data files must be .fits, .dat or .sub!\n");
            exit(-1);
        }
    }

    // Get some metafits metadata
    MetafitsMetadata *metafits_metadata = NULL;

    printf("\n\nExample of accessing Metafits Metadata:\n");

    if (mwalib_metafits_metadata_get(metafits_context, correlator_context, voltage_context, &metafits_metadata, error_message, ERROR_MESSAGE_LEN) == EXIT_SUCCESS)
    {
        // print a baseline
        printf("Baseline index 1: %ld vs %ld\n", metafits_metadata->baselines[1].ant1_index, metafits_metadata->baselines[1].ant2_index);

        // print an rfinput
        printf("RF Input index 1: ant index: %d, tile_id: %d name: %s pol: %s\n", metafits_metadata->rf_inputs[1].ant, metafits_metadata->rf_inputs[1].tile_id, metafits_metadata->rf_inputs[1].tile_name, metafits_metadata->rf_inputs[1].pol);

        // print a antenna
        printf("Ant index 1: %d name: %s elec len (m): %f\n", metafits_metadata->antennas[1].tile_id, metafits_metadata->antennas[1].tile_name, metafits_metadata->antennas[1].electrical_length_m);

        // print a coarse channel
        printf("Metafits Coarse channel index 1: receiver channel: %ld (centre = %f MHz)\n", metafits_metadata->metafits_coarse_chans[1].rec_chan_number, (float)metafits_metadata->metafits_coarse_chans[1].chan_centre_hz / 1000000.);

        // print a timestep
        printf("Metafits Timestep index 2: GPS Time = %f  (UNIX time: %f)\n", (double)metafits_metadata->metafits_timesteps[2].gps_time_ms / 1000., (double)metafits_metadata->metafits_timesteps[2].unix_time_ms / 1000.);

        // print the start time UTC and sched start unix time
        printf("Scheduled start time (UNIX): %f\n", metafits_metadata->sched_start_unix_time_ms / 1000.0);

        // Print the UTC value
        char utc_start_string[64];
        time_t time_utc_start = metafits_metadata->sched_start_utc;
        struct tm *utc_start_timeinfo = gmtime(&time_utc_start);
        const char date_format[] = "%c %Z";

        if (strftime(utc_start_string, sizeof(utc_start_string), date_format, utc_start_timeinfo) == 0)
        {
            printf("Error formatting sched_start_utc value.");
            return -1;
        }
        else
        {
            printf("Scheduled start time UTC: %s\n", utc_start_string);
        }

        // Clean up metadata
        mwalib_metafits_metadata_free(metafits_metadata);
    }
    else
    {
        printf("Error getting metafits metadata: %s\n", error_message);
    }

    // Clean up
    mwalib_metafits_context_free(metafits_context);
    mwalib_correlator_context_free(correlator_context);
    mwalib_voltage_context_free(voltage_context);

    free(error_message);

    return EXIT_SUCCESS;
}
