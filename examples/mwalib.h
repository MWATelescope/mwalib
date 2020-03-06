#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

/**
 * `mwalib` observation context. This is used to transport data out of gpubox
 * files and display info on the observation.
 *
 * The name is not following the rust convention of camel case, to make it look
 * more like a C library.
 */
typedef struct mwalibContext mwalibContext;

/**
 * # Safety
 * Free a previously-allocated float* (designed for use after
 * `mwalibContext_read_one_timestep_coarse_channel_bfp`).
 *
 * Python can't free memory itself, so this is useful for Python (and perhaps
 * other languages).
 */
void free_float_buffer(float *float_buffer_ptr, const long long *gpubox_hdu_size);

/**
 * # Safety
 * TODO: What does the caller need to know?
 * Display an `mwalibContext` struct.
 */
void mwalibContext_display(const mwalibContext *ptr);

/**
 * # Safety
 * TODO: What does the caller need to know?
 * Free a previously-allocated `mwalibContext` struct.
 */
void mwalibContext_free(mwalibContext *ptr);

/**
 * # Safety
 * TODO: What does the caller need to know?
 * Create an `mwalibContext` struct.
 */
mwalibContext *mwalibContext_new(const char *metafits, const char **gpuboxes, size_t gpubox_count);

/**
 * # Safety
 * Read MWA data.
 *
 * `num_scans` is an input and output variable. The input `num_scans` asks
 * `mwalib` to read in that many scans, but the output `num_scans` tells the
 * caller how many scans were actually read. This is done because the number of
 * scans requested might be more than what is available.
 *
 * `num_gpubox_files` and `gpubox_hdu_size` are output variables, allowing the
 * caller to know how to index the returned data.
 */
float *mwalibContext_read_one_timestep_coarse_channel_bfp(mwalibContext *context_ptr,
                                                          int *timestep_index,
                                                          int *coarse_channel_index);
