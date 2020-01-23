/**
 * @file gpubox_reader.c
 * @author Christopher H. Jordan
 * @date 08 Jan 2020
 * @brief This file helps to build a mwaObsContext_s struct (see process_args in
 * args.c), and with the same struct, facilitates the reading of raw MWA data.
 */

#include <stdlib.h>

#include "fitsreader.h"
#include "gpubox.h"

/**
 * @brief Deallocate the memory pointed within a mwaObsContext_s.
 * @param[in] obs Pointer to the mwaObsContext_s structure to be deallocated.
 * @returns EXIT_SUCCESS on success, EXIT_FAILURE on failure.
 */
int free_mwaObsContext(mwaObsContext_s *obs)
{
    free(obs->coarse_channels);
    free(obs->metafits_filename);
    free(obs->metafits_ptr);

    for (int i = 0; i < obs->gpubox_filename_count; i++) {
        free(obs->gpubox_filenames[i]);
        free(obs->gpubox_ptrs[i]);
    }
    free(obs->gpubox_filenames);
    free(obs->gpubox_ptrs);

    for (int i = 0; i < obs->gpubox_batch_count; i++) {
        // As we have already freed obs->gpubox_filenames and obs->gpubox_ptrs, we
        // don't need to free each element of obs->gpubox_filename_batches[i] and
        // obs->gpubox_ptr_batches[i]
        free(obs->gpubox_filename_batches[i]);
        free(obs->gpubox_ptr_batches[i]);
    }
    free(obs->gpubox_filename_batches);
    free(obs->gpubox_ptr_batches);

    return EXIT_SUCCESS;
}

/**
 * @brief Given a partially-populated mwaObsContext_s, determine the number of
 * fine channels in the observation.
 * @param[in] obs Pointer to the mwaObsContext_s structure to be populated. Must
 * have gpubox_filenames, gpubox_filename_count and gpubox_ptrs populated before
 * calling this function.
 * @param[inout] errorMessage Pointer to a string of length
 * MWALIB_ERROR_MESSAGE_LEN. The string is populated if there is an error,
 * otherwise, it isn't touched.
 * @returns EXIT_SUCCESS on success, EXIT_FAILURE on failure.
 */
int determine_gpubox_fine_channels(mwaObsContext_s *obs, char *errorMessage)
{
    // Determine the number of fine channels. Why isn't this in the metafits?
    // The following does assume that NAXIS2 is the same for all gpubox
    // files. But, this is a pretty reasonable assumption.

    // Move gpubox file 0 to HDU 2 (first HDU containing NAXIS2).
    if (move_to_fits_hdu(obs->gpubox_ptrs[0], 2, errorMessage) != EXIT_SUCCESS) {
        return EXIT_FAILURE;
    }
    int numFineChannels = 0;
    if (get_fits_int_value(obs->gpubox_ptrs[0], "NAXIS2", &numFineChannels, errorMessage) != EXIT_SUCCESS) {
        return EXIT_FAILURE;
    }
    // Move gpubox file 0 back to HDU 0, as all other gpubox files are there.
    if (move_to_fits_hdu(obs->gpubox_ptrs[0], 1, errorMessage) != EXIT_SUCCESS) {
        return EXIT_FAILURE;
    }

    return EXIT_SUCCESS;
}

/**
 * @brief Given a partially-populated mwaObsContext_s, determine the gpubox
 * batches. See gpubox.h for an explanation.
 * @param[in] obs Pointer to the mwaObsContext_s structure to be populated. Must
 * have gpubox_filenames, gpubox_filename_count and gpubox_ptrs populated before
 * calling this function.
 * @param[inout] errorMessage Pointer to a string of length
 * MWALIB_ERROR_MESSAGE_LEN. The string is populated if there is an error,
 * otherwise, it isn't touched.
 * @returns EXIT_SUCCESS on success, EXIT_FAILURE on failure.
 */
int determine_gpubox_batches(mwaObsContext_s *obs, char *errorMessage)
{
    // Determine the total number of batches. Try to read XX in
    // e.g. 1065880128_20131015134930_gpubox01_XX.fits - if that doesn't work,
    // we might have the "old format"
    // (i.e. 1065880128_20131015134930_gpubox01.fits).
    obs->gpubox_batch_count = 0;
    // format - 1 for old, 2 for new. We assume all data is of the same format.
    int format = 0;
    int obsid = 0, timestamp = 0;
    int batch = 0, dummy = 0;

    // Keep track of the filename batch counts so we don't have to loop as
    // often.
    int *batch_counts = (int *)calloc(sizeof(int), MWALIB_MAX_GPUBOX_BATCHES);

    // Loop over all files, and work out their batch numbers, so we know the
    // maximum batch number, as well as how many of each there are.
    for (int i = 0; i < obs->gpubox_filename_count; i++) {
        int count = sscanf(obs->gpubox_filenames[i], "%d_%d_gpubox%d_%d.fits", &obsid, &timestamp, &dummy, &batch);
        if (count != 4) {
            // Try the old format.
            count = sscanf(obs->gpubox_filenames[i], "%d_%d_gpubox%d.fits", &obsid, &timestamp, &dummy);
            if (count != 3) {
                snprintf(errorMessage, MWALIB_ERROR_MESSAGE_LEN, "Failed to determine the gpubox batch number for %s",
                         obs->gpubox_filenames[i]);
                return EXIT_FAILURE;
            }
        }
        // Batch numbers exist only when count == 4. Set obs->gpubox_batch_count
        // to the largest found batch number.
        if (count == 4 && batch > obs->gpubox_batch_count) {
            obs->gpubox_batch_count = batch;
        }
        batch_counts[batch]++;

        // For consistency checking, set the format, if it hasn't been set
        // already.
        if (format == 0) {
            if (count == 3) {
                format = 1;
            }
            else if (count == 4) {
                format = 2;
            }
        }

        // Check that the format is consistent with what has been previously
        // found with the other gpubox files.
        else if ((count == 3 && format != 1) || (count == 4 && format != 2)) {
            snprintf(errorMessage, MWALIB_ERROR_MESSAGE_LEN,
                     "The batch number format of %s disagrees with other gpubox files!", obs->gpubox_filenames[i]);
            return EXIT_FAILURE;
        }
    }
    // Increment the batch count, because e.g. if the biggest batch number is
    // 01, then we have two batches.
    obs->gpubox_batch_count++;

    // Now that we know how many batches there are, we can malloc new arrays for
    // each batch.
    obs->gpubox_filename_batches = (char ****)malloc(sizeof(char ***) * obs->gpubox_batch_count);
    obs->gpubox_ptr_batches = (fitsfile ****)malloc(sizeof(fitsfile ***) * obs->gpubox_batch_count);
    if (obs->gpubox_filename_batches == NULL) {
        snprintf(errorMessage, MWALIB_ERROR_MESSAGE_LEN, "malloc failed for obs->gpubox_filename_batches");
        return EXIT_FAILURE;
    }
    if (obs->gpubox_ptr_batches == NULL) {
        snprintf(errorMessage, MWALIB_ERROR_MESSAGE_LEN, "malloc failed for obs->gpubox_ptr_batches");
        return EXIT_FAILURE;
    }

    // Now allocate arrays for each individual batch. Also check that the batch
    // counts make sense - there should be an equal number of gpubox files in
    // each batch.
    for (int i = 0; i < obs->gpubox_batch_count; i++) {
        obs->gpubox_filename_batches[i] = (char ***)malloc(sizeof(char **) * batch_counts[i]);
        obs->gpubox_ptr_batches[i] = (fitsfile ***)malloc(sizeof(fitsfile **) * batch_counts[i]);
        if (obs->gpubox_filename_batches[i] == NULL) {
            snprintf(errorMessage, MWALIB_ERROR_MESSAGE_LEN,
                     "malloc failed for obs->gpubox_filename_batches[%d] (batch count = %d)", i, batch_counts[i]);
            return EXIT_FAILURE;
        }
        if (obs->gpubox_ptr_batches == NULL) {
            snprintf(errorMessage, MWALIB_ERROR_MESSAGE_LEN,
                     "malloc failed for obs->gpubox_ptr_batches[%d] (batch count = %d)", i, batch_counts[i]);
            return EXIT_FAILURE;
        }

        if (i > 0 && batch_counts[i] != batch_counts[0]) {
            snprintf(errorMessage, MWALIB_ERROR_MESSAGE_LEN,
                     "The batch number counts do not match! (%d for 00, %d for %02d)", batch_counts[0], batch_counts[i],
                     i);
            return EXIT_FAILURE;
        }
    }

    // Populate the new arrays. As this code is very similar to the above, we
    // relax the error checking. Use the existing batch_counts array to keep
    // track of how deep we are into each of the obs->gpubox_filename_batches
    // arrays.
    for (int i = 0; i < obs->gpubox_batch_count; i++) {
        batch_counts[i] = 0;
    }
    for (int i = 0; i < obs->gpubox_filename_count; i++) {
        int count = sscanf(obs->gpubox_filenames[i], "%d_%d_gpubox%d_%d.fits", &obsid, &timestamp, &dummy, &batch);
        if (count != 4) {
            // Try the old format.
            count = sscanf(obs->gpubox_filenames[i], "%d_%d_gpubox%d.fits", &obsid, &timestamp, &dummy);
        }
        obs->gpubox_filename_batches[batch][batch_counts[batch]] = &obs->gpubox_filenames[i];
        obs->gpubox_ptr_batches[batch][batch_counts[batch]] = &obs->gpubox_ptrs[i];
        batch_counts[batch]++;
    }
    free(batch_counts);

    return EXIT_SUCCESS;
}

/**
 * @brief Given a partially-populated mwaObsContext_s, determine the proper
 * start and end times of the observation. Probably necessary only for old MWA
 * correlator data.
 * @param[in] obs Pointer to the mwaObsContext_s structure to be populated. Must
 * have run determine_gpubox_batches before this function.
 * @param[inout] errorMessage Pointer to a string of length
 * MWALIB_ERROR_MESSAGE_LEN. The string is populated if there is an error,
 * otherwise, it isn't touched.
 * @returns EXIT_SUCCESS on success, EXIT_FAILURE on failure.
 */
int determine_obs_times(mwaObsContext_s *obs, char *errorMessage)
{
    // Determine the start and end times. gpubox filenames are not to be trusted
    // for this purpose.

    // Because gpubox files may not all start and end at the same time, anything
    // "dangling" is trimmed. e.g.
    // time:     0123456789abcdef
    // gpubox01: ################
    // gpubox02:  ###############
    // gpubox03: ################
    // gpubox04:   ##############
    // gpubox05: ###############
    // gpubox06: ################
    // Here, we start collecting data from time=2, and end at time=e, because
    // these are the first and last places that all gpubox files have data. All
    // other data is ignored.

    // As gpubox files can come in "batches" (e.g.
    // 1065880128_20131015134830_gpubox01_00.fits and
    // 1065880128_20131015134930_gpubox01_01.fits), we need to use the
    // first and last "batches" of gpubox files, too.

    // Deliberately overwrite anything that could be in the time variables.
    obs->start_time_milliseconds = 0;
    obs->end_time_milliseconds = 0;

    // Determine start and end times.
    long long this_start_time = 0;
    int this_start_milli_time = 0;
    long long this_end_time = 0;
    int this_end_milli_time = 0;
    for (int i = 0; i < obs->gpubox_filename_count / obs->gpubox_batch_count; i++) {
        // Start time.
        if (get_fits_long_long_value(*obs->gpubox_ptr_batches[0][i], "TIME", &this_start_time, errorMessage) !=
            EXIT_SUCCESS) {
            return EXIT_FAILURE;
        }
        if (get_fits_int_value(*obs->gpubox_ptr_batches[0][i], "MILLITIM", &this_start_milli_time, errorMessage) !=
            EXIT_SUCCESS) {
            return EXIT_FAILURE;
        }

        // Assign a new startTime, if the current gpubox file starts later than
        // anything we've already seen. Do comparisons only on ints. Scale the
        // value from time (which has units of seconds) by 1000 so that it now
        // is in milliseconds and can be neatly compared.
        this_start_time = this_start_time * 1000 + this_start_milli_time;
        if (this_start_time > obs->start_time_milliseconds) {
            obs->start_time_milliseconds = this_start_time;
        }

        // End time.

        // Determine the number of HDUs, so we can work out the end time of this
        // gpubox file.
        int hduCount = 0;
        if (get_fits_hdu_count(*obs->gpubox_ptr_batches[obs->gpubox_batch_count - 1][i], &hduCount, errorMessage) !=
            EXIT_SUCCESS) {
            return EXIT_FAILURE;
        }
        // Move to the last HDU, and grab the time. Note that move_to_fits_hdu
        // assumes that all HDU types are "0".
        if (move_to_fits_hdu(*obs->gpubox_ptr_batches[obs->gpubox_batch_count - 1][i], hduCount, errorMessage) !=
            EXIT_SUCCESS) {
            return EXIT_FAILURE;
        }
        if (get_fits_long_long_value(*obs->gpubox_ptr_batches[obs->gpubox_batch_count - 1][i], "TIME", &this_end_time,
                                     errorMessage) != EXIT_SUCCESS) {
            return EXIT_FAILURE;
        }
        if (get_fits_int_value(*obs->gpubox_ptr_batches[obs->gpubox_batch_count - 1][i], "MILLITIM",
                               &this_end_milli_time, errorMessage) != EXIT_SUCCESS) {
            return EXIT_FAILURE;
        }

        this_end_time = this_end_time * 1000 + this_end_milli_time;
        if (obs->end_time_milliseconds == 0 || this_end_time < obs->end_time_milliseconds) {
            obs->end_time_milliseconds = this_end_time;
        }

        // Move back to the first HDU.
        if (move_to_fits_hdu(*obs->gpubox_ptr_batches[obs->gpubox_batch_count - 1][i], 1, errorMessage) !=
            EXIT_SUCCESS) {
            return EXIT_FAILURE;
        }
    }

    return EXIT_SUCCESS;
}

/**
 *  @brief Given a memory limit (in gigabytes) and a mwalibObs struct,
 *  determine how many "scans" can be extracted from the gpubox files at a
 *  time. Here, "scan" refers to data containing visibilities from all baselines
 *  but only a single channel (aka fine channel).
 *  @param[in] Populated mwaObsContext_s struct.
 *  @param[in] The memory limit in gigabytes as an int.
 *  @param[inout] The number of scans that can be extracted from gpubox files
 *  within the memory limit.
 *  @param[inout] Pointer to an errorMessage.
 *  @returns EXIT_SUCCESS on success, or EXIT_FAILURE if there was an error.
 */
int determine_num_scans(mwaObsContext_s *obs, int memoryLimit, int *num_scans, char *errorMessage)
{
    // Get the number of fine-band channels from the metafits file. Use a long
    // type, just in case we have a lot.
    long num_chans;
    if (get_fits_long_value(obs->metafits_ptr, "NCHANS", &num_chans, errorMessage) != EXIT_SUCCESS) {
        return EXIT_FAILURE;
    }

    // Get the number of antennas (aka MWA tile) from the metafits file. This
    // can be found from the "number of inputs"; there are two inputs for each
    // antenna.
    int num_ants;
    if (get_fits_int_value(obs->metafits_ptr, "NINPUTS", &num_ants, errorMessage) != EXIT_SUCCESS) {
        return EXIT_FAILURE;
    }
    num_ants /= 2;

    // There are num_chans fine channels in each coarse band
    // (obs->gpubox_filename_count), and (num_ants+1)*num_ants/2 baselines, and
    // 4 polarisations. The product of all of this is the number of bytes needed
    // for a single scan.
    long long scan_size = num_chans * obs->gpubox_filename_count * (num_ants + 1) * num_ants / 2 * 4;

    // All that's left is to work out how many scans fit into the memory limit.
    *num_scans = memoryLimit * 1024 * 1024 * 1024 / scan_size;

    // If this value is less than 1 (i.e. 0), then not enough memory was
    // specified. cotter handles this by emitting a loud warning, and setting
    // the num_scans back to 1; we do the same here.
    if (num_scans == 0) {
        num_scans++;
    }

    return EXIT_SUCCESS;
}

/**
 *  @brief With the supplied and populated mwalibObs struct, read a number of
 *  scans from the gpubox files.

 *  @param[in] Populated mwaObsContext_s struct.
 *  @param[in] The memory limit in gigabytes as an int.
 *  @param[inout] The number of scans that can be extracted from gpubox files
 *  within the memory limit.
 *  @param[inout] Pointer to an errorMessage.
 *  @returns EXIT_SUCCESS on success, or EXIT_FAILURE if there was an error.
 */
/* int read_scans_from_gpubox_files(mwaObsContext_s *obs, int first_scan, int last_scan) */
/* { */
/* } */
