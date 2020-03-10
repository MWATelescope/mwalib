#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

typedef enum {
  /**
   * New correlator data (a.k.a. MWAX).
   */
  V2,
  /**
   * MWA raw data files with "gpubox" and batch numbers in their names.
   */
  Legacy,
  /**
   * gpubox files without any batch numbers.
   */
  OldLegacy,
} CorrelatorVersion;

/**
 * `mwalib` observation context. This is used to transport data out of gpubox
 * files and display info on the observation.
 *
 * The name is not following the rust convention of camel case, to make it look
 * more like a C library.
 */
typedef struct mwalibContext mwalibContext;

typedef struct {
  /**
   *
   * This is just a C struct to allow the caller to consume all of the metadata
   *
   * See definition of context::mwalibContext for full description of each attribute
   *
   *
   */
  unsigned int obsid;
  CorrelatorVersion corr_version;
  double coax_v_factor;
  unsigned long start_unix_time_milliseconds;
  unsigned long end_unix_time_milliseconds;
  unsigned long duration_milliseconds;
  size_t num_timesteps;
  size_t num_antennas;
  size_t num_baselines;
  unsigned long integration_time_milliseconds;
  size_t num_antenna_pols;
  size_t num_visibility_pols;
  size_t num_fine_channels_per_coarse;
  size_t num_coarse_channels;
  unsigned int fine_channel_width_hz;
  unsigned int coarse_channel_width_hz;
  unsigned int observation_bandwidth_hz;
  size_t timestep_coarse_channel_bytes;
  size_t num_gpubox_files;
  size_t timestep_coarse_channel_floats;
} mwalibMetadata;

typedef struct {
  unsigned long unix_time_ms;
} mwalibTimeStep;

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
 * Create and return a pointer to an `mwalibContext` struct or NULL if error occurs
 */
mwalibContext *mwalibContext_new(const char *metafits, const char **gpuboxes, size_t gpubox_count);

/**
 * # Safety
 * Read MWA data.
 *
 * This method takes as input a timestep_index and a coarse_channel_index to return one
 * HDU of data in [baseline][freq][pol][r][i] format
 */
float *mwalibContext_read_one_timestep_coarse_channel_bfp(mwalibContext *context_ptr,
                                                          int *timestep_index,
                                                          int *coarse_channel_index);

/**
 * # Safety
 * TODO: What does the caller need to know?
 * Free a previously-allocated `mwalibContext` struct.
 */
void mwalibMetadata_free(mwalibMetadata *ptr);

/**
 * This returns a struct containing the mwalibContext metadata
 * # Safety
 * TODO
 */
mwalibMetadata *mwalibMetadata_get(mwalibContext *ptr);

/**
 * # Safety
 * TODO: What does the caller need to know?
 * Free a previously-allocated `mwalibTimeStep` struct.
 */
void mwalibTimeStep_free(mwalibTimeStep *ptr);

/**
 * This returns a struct containing the requested timestep
 * Or NULL if there was an error
 * # Safety
 * TODO
 */
mwalibTimeStep *mwalibTimeStep_get(mwalibContext *ptr, size_t timestep_index);

/**
 * # Safety
 * Free a rust-allocated CString.
 *
 * mwalib uses error strings to detail the caller with anything that went
 * wrong. Non-rust languages cannot deallocate these strings; so, call this
 * function with the pointer to do that.
 */
void mwalib_free_rust_cstring(char *rust_cstring);
