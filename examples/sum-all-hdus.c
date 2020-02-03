// Given gpubox files, add their entire contents and report the sum.

#include <stdio.h>
#include <stdlib.h>

#include "fitsio.h"

int main(int argc, char *argv[]) {
    double sum = 0;

    fitsfile **fptr = malloc(sizeof(fitsfile *));
    float *buffer = malloc(sizeof(float));
    char *error_message = malloc(sizeof(char) * 2048);
    long *naxes = malloc(sizeof(long) * 2);
    int status = 0;
    int hdu_type = 0;
    long size = -1;
    char *first_floats = malloc(sizeof(char) * 2048);

    for (int i = 1; i < argc; i++) {
        printf("Reading %s\n", argv[i]);

        ffopen(fptr, argv[i], 0, &status);
        if (status) {
            printf("ffopen\n");
            fits_get_errstatus(status, error_message);
            printf("%s\n", error_message);
            return EXIT_FAILURE;
        }

        int hdu_index = 2;
        fits_movabs_hdu(fptr[0], hdu_index, &hdu_type, &status);
        if (status) {
            printf("fits_movabs_hdu\n");
            fits_get_errstatus(status, error_message);
            printf("%s\n", error_message);
            return EXIT_FAILURE;
        }

        double s = 0;
        while (1 == 1) {
            fits_get_img_size(fptr[0], 2, naxes, &status);
            if (status) {
                printf("fits_get_img_size\n");
                fits_get_errstatus(status, error_message);
                printf("%s\n", error_message);
                return EXIT_FAILURE;
            }
            if (naxes[0] * naxes[1] != size) {
                size = naxes[0] * naxes[1];
                free(buffer);
                buffer = malloc(sizeof(float) * size);
            }

            fits_read_img(fptr[0], TFLOAT, 1, size, 0, buffer, 0, &status);
            if (status) {
                printf("fits_read_img\n");
                fits_get_errstatus(status, error_message);
                printf("%s\n", error_message);
                return EXIT_FAILURE;
            }

            for (long long j = 0; j < size; j++) {
                if (j == 0 && hdu_index == 2) {
                    sprintf(first_floats, "First 5 floats: [%f, %f, %f, %f, %f]\n", buffer[j], buffer[j + 1],
                            buffer[j + 2], buffer[j + 3], buffer[j + 4]);
                }
                s += buffer[j];
            }

            hdu_index++;
            fits_movabs_hdu(fptr[0], hdu_index, 0, &status);
            if (status) {
                // Assume that we've hit the end.
                status = 0;
                break;
            }
        }
        printf("Sum: %f\n%s\n", s, first_floats);
        sum += s;
    }

    printf("Total sum: %f\n", sum);
    return EXIT_SUCCESS;
}
