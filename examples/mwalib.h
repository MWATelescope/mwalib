#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

/**
 * `mwalib` buffer. Used to transport data out of gpubox files.
 *
 * The name is not following the rust convention of camel case, to make it look
 * more like a C library.
 */
typedef struct mwalibBuffer mwalibBuffer;

/**
 * `mwalib` observation context.
 *
 * The name is not following the rust convention of camel case, to make it look
 * more like a C library.
 */
typedef struct mwalibObsContext mwalibObsContext;

/**
 * Free an `mwalibBuffer` struct.
 */
void mwalibBuffer_free(mwalibBuffer *mwalib_buffer_ptr);

/**
 * Create an `mwalibBuffer` struct.
 */
mwalibBuffer *mwalibBuffer_new(const mwalibObsContext *context_ptr, size_t num_scans);

/**
 * Read MWA data.
 */
float ***mwalibBuffer_read(const mwalibObsContext *context_ptr,
                           mwalibBuffer *mwalib_buffer_ptr,
                           int *num_scans,
                           int *num_gpubox_files,
                           long long *gpubox_hdu_size);

/**
 * Display an `mwalibObsContext` struct.
 */
void mwalibObsContext_display(const mwalibObsContext *ptr);

/**
 * Free a previously-allocated `mwalibObsContext` struct.
 */
void mwalibObsContext_free(mwalibObsContext *ptr);

/**
 * Create an `mwalibObsContext` struct.
 */
mwalibObsContext *mwalibObsContext_new(const char *metafits,
                                       const char **gpuboxes,
                                       size_t gpubox_count);
