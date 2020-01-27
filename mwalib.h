#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

typedef struct mwalibObsContext mwalibObsContext;

/**
 * Display an `mwalibObsContext` struct.
 */
void mwalibObsContext_display(mwalibObsContext *ptr);

/**
 * Free a previously-allocated `mwalibObsContext` struct.
 */
void mwalibObsContext_free(mwalibObsContext *ptr);

/**
 * Create an `mwalibObsContext` struct.
 */
mwalibObsContext *mwalibObsContext_new(char *metafits, char **gpuboxes, size_t gpubox_count);
