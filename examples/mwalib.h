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
 * Display an `mwalibContext` struct.
 */
void mwalibContext_display(const mwalibContext *ptr);

/**
 * Free a previously-allocated `mwalibContext` struct.
 */
void mwalibContext_free(mwalibContext *ptr);

/**
 * Create an `mwalibContext` struct.
 */
mwalibContext *mwalibContext_new(const char *metafits, const char **gpuboxes, size_t gpubox_count);

/**
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
float ***mwalibContext_read(mwalibContext *context_ptr,
                            int *num_scans,
                            int *num_gpubox_files,
                            long long *gpubox_hdu_size);
