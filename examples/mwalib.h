#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

/**
 * Enum for all of the known variants of file format based on Correlator version
 *
 */
typedef enum {
  /**
   * MWAX correlator (v2.0)
   */
  V2,
  /**
   * MWA correlator (v1.0), having data files with "gpubox" and batch numbers in their names.
   */
  Legacy,
  /**
   * MWA correlator (v1.0), having data files without any batch numbers.
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

/**
 * Representation in C of an mwalibAntenna struct
 */
typedef struct {
  /**
   * See definition of context::mwalibAntenna for full description of each attribute
   */
  uint32_t antenna;
  uint32_t tile_id;
  char *tile_name;
} mwalibAntenna;

/**
 * Representation in C of an mwalibCoarseChannel struct
 */
typedef struct {
  /**
   * See definition of context::mwalibContext for full description of each attribute
   */
  uintptr_t correlator_channel_number;
  uintptr_t receiver_channel_number;
  uintptr_t gpubox_number;
  uint32_t channel_width_hz;
  uint32_t channel_start_hz;
  uint32_t channel_centre_hz;
  uint32_t channel_end_hz;
} mwalibCoarseChannel;

/**
 *
 * This a C struct to allow the caller to consume all of the metadata
 *
 */
typedef struct {
  /**
   * See definition of context::mwalibContext for full description of each attribute
   */
  uint32_t obsid;
  CorrelatorVersion corr_version;
  double coax_v_factor;
  uint64_t start_unix_time_milliseconds;
  uint64_t end_unix_time_milliseconds;
  uint64_t duration_milliseconds;
  uintptr_t num_timesteps;
  uintptr_t num_antennas;
  uintptr_t num_baselines;
  uintptr_t num_rf_inputs;
  uintptr_t num_antenna_pols;
  uintptr_t num_visibility_pols;
  uintptr_t num_coarse_channels;
  uint64_t integration_time_milliseconds;
  uint32_t fine_channel_width_hz;
  uint32_t observation_bandwidth_hz;
  uint32_t coarse_channel_width_hz;
  uintptr_t num_fine_channels_per_coarse;
  uintptr_t num_timestep_coarse_channel_bytes;
  uintptr_t num_timestep_coarse_channel_floats;
  uintptr_t num_gpubox_files;
} mwalibMetadata;

/**
 * Representation in C of an mwalibRFInput struct
 */
typedef struct {
  /**
   * See definition of context::mwalibContext for full description of each attribute
   */
  uint32_t input;
  uint32_t antenna;
  uint32_t tile_id;
  char *tile_name;
  char *pol;
  double electrical_length_m;
  double north_m;
  double east_m;
  double height_m;
  uint32_t vcs_order;
  uint32_t subfile_order;
  bool flagged;
} mwalibRFInput;

/**
 *
 * C Representation of a mwalibTimeStep struct
 *
 */
typedef struct {
  /**
   * See definition of context::mwalibTimeStep for full description of each attribute
   */
  uint64_t unix_time_ms;
} mwalibTimeStep;

/**
 * Free a previously-allocated `mwalibAntenna` struct.
 *
 * # Arguments
 *
 * * `antenna_ptr` - pointer to an already populated mwalibAntenna object
 *
 *
 * # Returns
 *
 * * Nothing
 *
 *
 * # Safety
 * * This must be called once caller is finished with the mwalibAntenna object
 * * antenna_ptr must point to a populated mwalibAntenna object from the mwalibAntenna_new function.
 * * antenna_ptr must not have already been freed.
 */
void mwalibAntenna_free(mwalibAntenna *antenna_ptr);

/**
 * This returns a struct containing the requested antenna
 *
 * # Arguments
 *
 * * `context_ptr` - pointer to an already populated mwalibContext object.
 *
 * * `antenna_index` - item in the antenna array to return. This must be be between 0 and context->num_antennas - 1.
 *
 * * `error_message` - pointer to already allocated buffer for any error messages to be returned to the caller.
 *
 * * `error_message_length` - length of error_message char* buffer.
 *
 *
 * # Returns
 *
 * * A Rust-owned populated mwalibAntenna struct or NULL if there was an error (check error_message)
 *
 *
 * # Safety
 * * error_message *must* point to an already allocated char* buffer for any error messages.
 * * context_ptr must point to a populated mwalibContext object from the mwalibContext_new function.
 * * Caller must call mwalibAntenna_free once finished, to free the rust memory.
 */
mwalibAntenna *mwalibAntenna_get(mwalibContext *context_ptr,
                                 size_t antenna_index,
                                 uint8_t *error_message,
                                 size_t error_message_length);

/**
 * Free a previously-allocated `mwalibCoarseChannel` struct.
 *
 * # Arguments
 *
 * * `coarse_channel_ptr` - pointer to an already populated mwalibCoarseChannel object
 *
 *
 * # Returns
 *
 * * Nothing
 *
 *
 * # Safety
 * * This must be called once caller is finished with the mwalibCoarseChannel object
 * * coarse_channel_ptr must point to a populated mwalibCoarseChannel object from the mwalibCoarseChannel_new function.
 * * coarse_channel_ptr must not have already been freed.
 */
void mwalibCoarseChannel_free(mwalibCoarseChannel *coarse_channel_ptr);

/**
 * This returns a struct containing the requested coarse channel
 *
 * # Arguments
 *
 * * `context_ptr` - pointer to an already populated mwalibContext object.
 *
 * * `coarse_channel_index` - item in the coarse_channel array to return. This must be be between 0 and context->num_coarse_channels - 1.
 *
 * * `error_message` - pointer to already allocated buffer for any error messages to be returned to the caller.
 *
 * * `error_message_length` - length of error_message char* buffer.
 *
 *
 * # Returns
 *
 * * A Rust-owned populated mwalibCoarseChannel struct or NULL if there was an error (check error_message)
 *
 *
 * # Safety
 * * error_message *must* point to an already allocated char* buffer for any error messages.
 * * context_ptr must point to a populated mwalibContext object from the mwalibContext_new function.
 * * Caller must call mwalibCoarseChannel_free once finished, to free the rust memory.
 */
mwalibCoarseChannel *mwalibCoarseChannel_get(mwalibContext *context_ptr,
                                             size_t coarse_channel_index,
                                             uint8_t *error_message,
                                             size_t error_message_length);

/**
 * Display an `mwalibContext` struct.
 *
 *
 * # Arguments
 *
 * * `context_ptr` - pointer to an already populated mwalibContext object
 *
 * * `error_message` - pointer to already allocated buffer for any error messages to be returned to the caller.
 *
 * * `error_message_length` - length of error_message char* buffer.
 *
 *
 * # Returns
 *
 * * 0 on success, 1 on failure
 *
 *
 * # Safety
 * * error_message *must* point to an already allocated char* buffer for any error messages.
 * * context_ptr must contain an mwalibContext object already populated via mwalibContext_new
 */
int32_t mwalibContext_display(const mwalibContext *context_ptr,
                              uint8_t *error_message,
                              size_t error_message_length);

/**
 * Free a previously-allocated `mwalibContext` struct.
 *
 * # Arguments
 *
 * * `context_ptr` - pointer to an already populated mwalibContext object
 *
 *
 * # Returns
 *
 * * Nothing
 *
 *
 * # Safety
 * * This must be called once caller is finished with the mwalibContext object
 * * context_ptr must point to a populated mwalibContext object from the mwalibContext_new function.
 * * context_ptr must not have already been freed.
 */
void mwalibContext_free(mwalibContext *context_ptr);

/**
 * Free a previously-allocated float* created by mwalibContext_read_by_baseline.
 *
 * Python can't free memory itself, so this is useful for Python (and perhaps
 * other languages).
 *
 * # Arguments
 *
 * * `float_buffer_ptr` - pointer to an already populated float buffer object.
 *
 * * `float_buffer_len` - length of float buffer.
 *
 *
 * # Returns
 *
 * * Nothing
 *
 *
 * # Safety
 * * This must be called once caller is finished with the float buffer object
 * * float_buffer_ptr must point to a populated float buffer from the mwalibContext_read_by_baseline function.
 * * float_buffer_ptr must not have already been freed.
 */
void mwalibContext_free_read_buffer(float *float_buffer_ptr,
                                    const long long *float_buffer_len);

/**
 * Create and return a pointer to an `mwalibContext` struct
 *
 * # Arguments
 *
 * * `metafits` - pointer to char* buffer containing the full path and filename of a metafits file.
 *
 * * `gpuboxes` - pointer to array of char* buffers containing the full path and filename of the gpubox FITS files.
 *
 * * `gpubox_count` - length of the gpubox char* array.
 *
 * * `error_message` - pointer to already allocated buffer for any error messages to be returned to the caller.
 *
 * * `error_message_length` - length of error_message char* buffer.
 *
 *
 * # Returns
 *
 * * A Rust-owned populated `mwalibContext` struct or NULL if there was an error (check error_message)
 *
 *
 * # Safety
 * * error_message *must* point to an already allocated `char*` buffer for any error messages.
 * * Caller *must* call the appropriate _free function to release the rust memory.
 */
mwalibContext *mwalibContext_get(const char *metafits,
                                 const char **gpuboxes,
                                 size_t gpubox_count,
                                 uint8_t *error_message,
                                 size_t error_message_length);

/**
 * Read a single timestep / coarse channel of MWA data.
 *
 * This method takes as input a timestep_index and a coarse_channel_index to return one
 * HDU of data in [baseline][freq][pol][r][i] format
 *
 * # Arguments
 *
 * * `context_ptr` - pointer to an already populated mwalibContext object.
 *
 * * `timestep_index` - index within the timestep array for the desired timestep. This corresponds
 *                      to mwalibTimeStep.get(context, N) where N is timestep_index.
 *
 * * `coarse_channel_index` - index within the coarse_channel array for the desired coarse channel. This corresponds
 *                            to mwalibCoarseChannel.get(context, N) where N is coarse_channel_index.
 *
 * * `buffer_ptr` - pointer to caller-owned and allocated buffer to write data into.
 *
 * * `buffer_len` - length of `buffer_ptr`.
 *
 * * `error_message` - pointer to already allocated buffer for any error messages to be returned to the caller.
 *
 * * `error_message_length` - length of error_message char* buffer.
 *
 *
 * # Returns
 *
 * * 0 on success, 1 on failure
 *
 *
 * # Safety
 * * error_message *must* point to an already allocated char* buffer for any error messages.
 * * context_ptr must point to a populated object from the mwalibContext_new function.
 * * Caller *must* call mwalibContext_free_read_buffer function to release the rust memory.
 */
int32_t mwalibContext_read_by_baseline(mwalibContext *context_ptr,
                                       uintptr_t timestep_index,
                                       uintptr_t coarse_channel_index,
                                       float *buffer_ptr,
                                       size_t buffer_len,
                                       uint8_t *error_message,
                                       size_t error_message_length);

/**
 * Read a single timestep / coarse channel of MWA data.
 *
 * This method takes as input a timestep_index and a coarse_channel_index to return one
 * HDU of data in [freq][baseline][pol][r][i] format
 *
 * # Arguments
 *
 * * `context_ptr` - pointer to an already populated mwalibContext object.
 *
 * * `timestep_index` - index within the timestep array for the desired timestep. This corresponds
 *                      to mwalibTimeStep.get(context, N) where N is timestep_index.
 *
 * * `coarse_channel_index` - index within the coarse_channel array for the desired coarse channel. This corresponds
 *                            to mwalibCoarseChannel.get(context, N) where N is coarse_channel_index.
 *
 * * `buffer_ptr` - pointer to caller-owned and allocated buffer to write data into.
 *
 * * `buffer_len` - length of `buffer_ptr`.
 *
 * * `error_message` - pointer to already allocated buffer for any error messages to be returned to the caller.
 *
 * * `error_message_length` - length of error_message char* buffer.
 *
 *
 * # Returns
 *
 * * 0 on success, 1 on failure
 *
 *
 * # Safety
 * * error_message *must* point to an already allocated char* buffer for any error messages.
 * * context_ptr must point to a populated object from the mwalibContext_new function.
 * * Caller *must* call mwalibContext_free_read_buffer function to release the rust memory.
 */
int32_t mwalibContext_read_by_frequency(mwalibContext *context_ptr,
                                        uintptr_t timestep_index,
                                        uintptr_t coarse_channel_index,
                                        float *buffer_ptr,
                                        size_t buffer_len,
                                        uint8_t *error_message,
                                        size_t error_message_length);

/**
 * Free a previously-allocated `mwalibContext` struct.
 *
 * # Arguments
 *
 * * `metadata_ptr` - pointer to an already populated mwalibMetadata object
 *
 *
 * # Returns
 *
 * * Nothing
 *
 *
 * # Safety
 * * This must be called once caller is finished with the mwalibMetadata object
 * * metadata_ptr must point to a populated mwalibMetadata object from the mwalibMetadata_new function.
 * * metadata_ptr must not have already been freed.
 */
void mwalibMetadata_free(mwalibMetadata *metadata_ptr);

/**
 * This returns a struct containing the mwalibContext metadata
 *
 * # Arguments
 *
 * * `context_ptr` - pointer to an already populated mwalibContext object.
 *
 * * `error_message` - pointer to already allocated buffer for any error messages to be returned to the caller.
 *
 * * `error_message_length` - length of error_message char* buffer.
 *
 *
 * # Returns
 *
 * * A Rust-owned populated mwalibMetadata struct or NULL if there was an error (check error_message)
 *
 *
 * # Safety
 * * error_message *must* point to an already allocated char* buffer for any error messages.
 * * context_ptr must point to a populated mwalibContext object from the mwalibContext_new function.
 * * Caller must call mwalibMetadata_free once finished, to free the rust memory.
 */
mwalibMetadata *mwalibMetadata_get(mwalibContext *context_ptr,
                                   uint8_t *error_message,
                                   size_t error_message_length);

/**
 * Free a previously-allocated `mwalibRFInput` struct.
 *
 * # Arguments
 *
 * * `rf_input_ptr` - pointer to an already populated mwalibRFInput object
 *
 *
 * # Returns
 *
 * * Nothing
 *
 *
 * # Safety
 * * This must be called once caller is finished with the mwalibRFInput object
 * * rf_input_ptr must point to a populated mwalibRFInput object from the mwalibRFInput_new function.
 * * rf_input_ptr must not have already been freed.
 */
void mwalibRFInput_free(mwalibRFInput *rf_input_ptr);

/**
 * This returns a struct containing the requested antenna
 * Or NULL if there was an error
 *
 * # Arguments
 *
 * * `context_ptr` - pointer to an already populated mwalibContext object.
 *
 * * `rf_input_index` - item in the rf_input array to return. This must be be between 0 and context->num_rf_inputs - 1.
 *
 * * `error_message` - pointer to already allocated buffer for any error messages to be returned to the caller.
 *
 * * `error_message_length` - length of error_message char* buffer.
 *
 *
 * # Returns
 *
 * * A Rust-owned populated mwalibRFInput struct or NULL if there was an error (check error_message)
 *
 *
 * # Safety
 * * error_message *must* point to an already allocated char* buffer for any error messages.
 * * context_ptr must point to a populated mwalibContext object from the mwalibContext_new function.
 * * Caller must call mwalibRFInput_free once finished, to free the rust memory.
 */
mwalibRFInput *mwalibRFInput_get(mwalibContext *context_ptr,
                                 size_t rf_input_index,
                                 uint8_t *error_message,
                                 size_t error_message_length);

/**
 * Free a previously-allocated `mwalibTimeStep` struct.
 *
 * # Arguments
 *
 * * `timestep_ptr` - pointer to an already populated mwalibTimeStep object
 *
 *
 * # Returns
 *
 * * Nothing
 *
 *
 * # Safety
 * * This must be called once caller is finished with the mwalibTimeStep object
 * * timestep_ptr must point to a populated mwalibTimeStep object from the mwalibTimeStep_new function.
 * * timestep_ptr must not have already been freed.
 */
void mwalibTimeStep_free(mwalibTimeStep *timestep_ptr);

/**
 * This returns a struct containing the requested timestep
 *
 * # Arguments
 *
 * * `context_ptr` - pointer to an already populated mwalibContext object.
 *
 * * `timestep_index` - item in the timestep array to return. This must be be between 0 and context->num_timesteps - 1.
 *
 * * `error_message` - pointer to already allocated buffer for any error messages to be returned to the caller.
 *
 * * `error_message_length` - length of error_message char* buffer.
 *
 *
 * # Returns
 *
 * * A Rust-owned populated mwalibTimeStep struct or NULL if there was an error (check error_message)
 *
 *
 * # Safety
 * * error_message *must* point to an already allocated char* buffer for any error messages.
 * * context_ptr must point to a populated mwalibContext object from the mwalibContext_new function.
 * * Caller must call mwalibTimeStep_free once finished, to free the rust memory.
 */
mwalibTimeStep *mwalibTimeStep_get(mwalibContext *context_ptr,
                                   size_t timestep_index,
                                   uint8_t *error_message,
                                   size_t error_message_length);

/**
 * Free a rust-allocated CString.
 *
 * mwalib uses error strings to detail the caller with anything that went
 * wrong. Non-rust languages cannot deallocate these strings; so, call this
 * function with the pointer to do that.
 *
 * # Arguments
 *
 * * `rust_cstring` - pointer to a `char*` of a Rust string
 *
 *
 * # Returns
 *
 * * Nothing
 *
 * # Safety
 * * rust_cstring must not have already been freed and must point to a Rust string.
 */
void mwalib_free_rust_cstring(char *rust_cstring);
