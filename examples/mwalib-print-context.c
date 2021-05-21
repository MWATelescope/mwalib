/*!
Example code to print context info, given at least a metafits file and optionally and one or more gpubox files or voltage data files
*/
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "mwalib.h"

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
        if (mwalib_metafits_context_new(argv[1], CorrLegacy, &metafits_context, error_message, ERROR_MESSAGE_LEN) != EXIT_SUCCESS)
        {
            printf("Error getting metaafits context: %s\n", error_message);
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

            free(files);
        }
        else
        {
            // Unknown files!
            printf("Error- provided data files must be .fits, .dat or .sub!\n");
            exit(-1);
        }
    }

    // Clean up
    mwalib_metafits_context_free(metafits_context);
    mwalib_correlator_context_free(correlator_context);
    mwalib_voltage_context_free(voltage_context);

    free(error_message);

    return EXIT_SUCCESS;
}
