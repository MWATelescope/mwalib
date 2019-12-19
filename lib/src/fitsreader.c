/**
 * @file fitsreader.h
 * @author Greg Sleap
 * @date 18 Dec 2019
 * @brief Functions to read the metafits file
 *
 */
#include <fitsio.h>
#include <stdlib.h>
#include <string.h>
#include "fitsreader.h"
#include "global.h"

/**
 *
 *  @brief Opens a fits file for reading. 
 *  @param[in,out] fptr Pointer to a pointer of the openned fits file.
 *  @param[in] filename Full path/name of the file to be openned.
 *  @param[inout] errorMessage Pointer to a string of length ARG_ERROR_MESSAGE_LEN containing an error message or empty string if no error
 *  @returns EXIT_SUCCESS on success, or EXIT_FAILURE if there was an error.
 */
int open_fits(fitsfile **fptr, const char *filename, char* errorMessage)
{  
    // Initialise errorMessage
    strncpy(errorMessage, "", MWALIB_ERROR_MESSAGE_LEN);    
    int status = 0;

    if (fits_open_file(fptr, filename, READONLY, &status))
    {    
        fits_get_errstatus(status, errorMessage);    
        return EXIT_FAILURE;
    }

  return EXIT_SUCCESS;
}

/**
 *
 *  @brief Closes the fits file.
 *  @param[in,out] fptr Pointer to a pointer to the fitsfile structure.
 *  @returns EXIT_SUCCESS on success, or EXIT_FAILURE if there was an error.
 */
int close_fits(fitsfile **fptr)
{  
  int status = 0;

  if (*fptr != NULL)
  {
    if (fits_close_file(*fptr, &status))
    {
      char error_text[30]="";
      fits_get_errstatus(status, error_text);      
      return EXIT_FAILURE;
    }
    else
    {
      *fptr = NULL;
    }
  }
  
  return(EXIT_SUCCESS);
}

int get_fits_string_value(fitsfile* fptr, char* keyword, char* value, char* errorMessage)
{
    int status = 0;
        
    if ( fits_read_key(fptr, TSTRING, keyword, value, NULL, &status) )
    {        
        fits_get_errstatus(status, errorMessage);
        sprintf(errorMessage, "%s (%s)", errorMessage, keyword);        
        return EXIT_FAILURE;
    }
        
    return EXIT_SUCCESS;
}

int get_fits_int_value(fitsfile* fptr, char* keyword, int* value, char* errorMessage)
{
    char string_value[100] = {""};

    if (get_fits_string_value(fptr, keyword, string_value, errorMessage) == EXIT_SUCCESS)
    {
        *value = atoi(string_value);
        return EXIT_SUCCESS;
    }
    else
    {
        return EXIT_FAILURE;
    }    
}

int get_fits_long_value(fitsfile* fptr, char* keyword, long* value, char* errorMessage)
{
    char string_value[100] = {""};

    if (get_fits_string_value(fptr, keyword, string_value, errorMessage) == EXIT_SUCCESS)
    {
        *value = atol(string_value);
        return EXIT_SUCCESS;
    }
    else
    {
        return EXIT_FAILURE;
    }    
}

int get_fits_float_value(fitsfile* fptr, char* keyword, float* value, char* errorMessage)
{
    char string_value[100] = {""};

    if (get_fits_string_value(fptr, keyword, string_value, errorMessage) == EXIT_SUCCESS)
    {
        *value = atof(string_value);
        return EXIT_SUCCESS;
    }
    else
    {
        return EXIT_FAILURE;
    }    
}

int get_fits_double_value(fitsfile* fptr, char* keyword, double* value, char* errorMessage)
{
    char string_value[100] = {""};

    if (get_fits_string_value(fptr, keyword, string_value, errorMessage) == EXIT_SUCCESS)
    {
        *value = atof(string_value);
        return EXIT_SUCCESS;
    }
    else
    {
        return EXIT_FAILURE;
    }    
}