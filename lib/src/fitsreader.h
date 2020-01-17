/**
 * @file fitsreader.h
 * @author Greg Sleap
 * @date 18 Dec 2019
 * @brief Definitions for functions to read fits files
 *
 */
#pragma once
#include <fitsio.h>

int open_fits(fitsfile **fptr, const char *filename, char *errorMessage);
int close_fits(fitsfile **fptr);

int get_fits_hdu_count(fitsfile *fptr, int *hduCount, char *errorMessage);
int move_to_fits_hdu(fitsfile *fptr, int hduNum, char *errorMessage);

int get_fits_int_value(fitsfile *fptr, char *keyword, int *value, char *errorMessage);
int get_fits_long_value(fitsfile *fptr, char *keyword, long *value, char *errorMessage);
int get_fits_long_long_value(fitsfile *fptr, char *keyword, long long *value, char *errorMessage);
int get_fits_float_value(fitsfile *fptr, char *keyword, float *value, char *errorMessage);
int get_fits_double_value(fitsfile *fptr, char *keyword, double *value, char *errorMessage);
int get_fits_string_value(fitsfile *fptr, char *keyword, char *value, char *errorMessage);
int get_fits_comma_delimited_ints(fitsfile *fptr, char *keyword, int string_size, char *string, int *int_count, int *int_array, char *errorMessage);
