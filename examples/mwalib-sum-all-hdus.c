// Given gpubox files, add their entire contents and report the sum.

#include <stdio.h>
#include <stdlib.h>

#include "fitsio.h"
#include "mwalib.h"

int main(int argc, char *argv[]) {
    // Assume that the first file provided is the metafits file, and all others
    // are gpubox files. Therefore, we need at least two files provided to main,
    // such that there's at least one gpubox file.
    if (argc < 3) {
        printf("At least two files are needed.\n");
        return EXIT_FAILURE;
    }

    const char **gpuboxes = malloc(sizeof(char *) * (argc - 2));
    for (int i = 0; i < argc - 2; i++) {
        gpuboxes[i] = argv[i + 2];
    }

    int num_scans = 10;
    mwalibContext *context = mwalibContext_new(argv[1], gpuboxes, argc - 2);

    int num_gpubox_files = 0;
    long long gpubox_hdu_size = 0;
    float ***data_buffer = NULL;

    double sum = 0;

    while (num_scans > 0) {
        data_buffer = mwalibContext_read(context, &num_scans, &num_gpubox_files, &gpubox_hdu_size);
        if (data_buffer != NULL) {
            for (int scan = 0; scan < num_scans; scan++) {
                for (int gpubox = 0; gpubox < num_gpubox_files; gpubox++) {
                    for (long long i = 0; i < gpubox_hdu_size; i++) {
                        sum += data_buffer[scan][gpubox][i];
                    }
                    free(data_buffer[scan][gpubox]);
                }
                free(data_buffer[scan]);
            }
            free(data_buffer);
        }
    }

    printf("Total sum: %f\n", sum);

    mwalibContext_free(context);
    free(gpuboxes);

    return EXIT_SUCCESS;
}
