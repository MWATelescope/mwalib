#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

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

    mwalibObsContext *context = mwalibObsContext_new(argv[1], gpuboxes, argc - 2);
    mwalibObsContext_display(context);
    mwalibObsContext_free(context);

    free(gpuboxes);

    return EXIT_SUCCESS;
}
