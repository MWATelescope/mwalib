/**
 * @file main.c
 * @author Greg Sleap
 * @date 19 Dec 2019
 * @brief Main test harness code for mwalib_test
 *
 */
#include <getopt.h>
#include <linux/limits.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "../../lib/src/args.h"
#include "../../lib/src/global.h"

/**
 *
 *  @brief This function prints the command line usage text - equivalent to --help
 *  @returns EXIT_SUCCESS on success, or anything else if there was an error.
 */
int print_usage()
{
    printf("Usage:\n\n");
    printf("mwalib_test -m FILENAME GPUBOXFILE [GPUBOXFILE]...\n\n");
    printf("-m --metafits FILENAME Full path to metafits file\n");
    printf("GPUBOXFILENAME Full path to each gpubox or mwax FITS file\n");
    return EXIT_SUCCESS;
}

/**
 *
 *  @brief This processes command line arguments and passes them to mwalib
 *  @param[in] argc Count of command line args
 *  @param[in] argv Array of command line args
 *  @returns EXIT_SUCCESS on success, or anything else if there was an error.
 */
int main(int argc, char *argv[])
{
    char error[MWALIB_ERROR_MESSAGE_LEN];
    printf("mwalib Test Harness\n");

    // Variables for command line arguments
    char *metafits_filename = NULL;
    char *gpubox_filenames[MWALIB_MAX_GPUBOX_FILENAMES] = {NULL};

    // Process command line arguments
    static const char *optString = "m:?";

    // clang-format off
    static const struct option longOpts[] = {
        { "metafits", required_argument, NULL, 'm' },
        { "help", no_argument, NULL, '?' },
        { NULL, no_argument, NULL, 0 }
    };
    // clang-format on

    int opt = 0;
    int longIndex = 0;
    int i = 2; // Start at 2 to skip argv[0] and argv[1]
    int gpuboxCount = 0;

    while (i < argc) {
        if ((opt = getopt_long(argc, argv, optString, longOpts, &longIndex)) != -1) {
            /* Options */
            switch (opt) {
            case 'm':
                metafits_filename = optarg;
                break;

            case '?':
                print_usage();
                return EXIT_FAILURE;

            default:
                /* You won't actually get here. */
                break;
            }
        }
        else {
            /* Positional arguments for gpubox filenames */
            gpubox_filenames[gpuboxCount] = argv[i];
            gpuboxCount++;
        }

        i++; // Skip to the next argument
    }

    // Initialise mwalibArgs struct
    mwalibArgs_s args;

    initialise_args(&args);

    //
    // Copy command line args for use by process args in the mwalib library
    //

    // metafits filename
    if (metafits_filename) {
        if (set_metafits_filename(&args, metafits_filename) != EXIT_SUCCESS) {
            printf("Error setting metafits file: %s\n", metafits_filename);
            return 1;
        }

        printf("Metafits file: %s\n", args.metafits_filename);
    }

    // gpubox filenames
    if (gpuboxCount > 0) {
        printf("GPUbox files: %d\n", gpuboxCount);

        for (int i = 0; i < gpuboxCount; i++) {
            if (add_gpubox_filename(&args, gpubox_filenames[i]) != EXIT_SUCCESS) {
                printf("Error adding gpubox file: %s\n", gpubox_filenames[i]);
                return 1;
            }

            printf("%s\n", args.gpubox_filenames[args.gpubox_filename_count]);
        }
        printf("\n");
    }

    // Check and parse what we have given the mwalib library
    printf("Processing passed in arguments...\n");
    if (process_args(&args, error) != EXIT_SUCCESS) {
        printf("Error: %s\n", error);
        print_usage();
        return 1;
    }

    printf("Observation ID: %d\n", args.obs_id);

    printf("Success!\n");
    return EXIT_SUCCESS;
}
