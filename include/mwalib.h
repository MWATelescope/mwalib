#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

/**
 * The MWA's latitude on Earth in radians. This is -26d42m11.94986s.
 */
#define MWA_LATITUDE_RADIANS -0.4660608448386394

/**
 * The MWA's longitude on Earth in radians. This is 116d40m14.93485s.
 */
#define MWA_LONGITUDE_RADIANS 2.0362898668561042

/**
 * The MWA's altitude in metres.
 */
#define MWA_ALTITUDE_METRES 377.827

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
 *
 * `mwalib` correlator observation context. This represents the basic metadata for a correlator observation.
 *
 */
typedef struct CorrelatorContext CorrelatorContext;

/**
 * `mwalib` metafits context. This represents the basic metadata for the observation.
 *
 */
typedef struct MetafitsContext MetafitsContext;

/**
 *
 * `mwalib` voltage captue system (VCS) observation context. This represents the basic metadata for a voltage capture observation.
 *
 */
typedef struct VoltageContext VoltageContext;

/**
 *
 * This a C struct to allow the caller to consume the metafits metadata
 *
 */
typedef struct {
  /**
   * Observation id
   */
  uint32_t obsid;
  /**
   * Latitude of centre point of MWA in raidans
   */
  double mwa_latitude_radians;
  /**
   * Longitude of centre point of MWA in raidans
   */
  double mwa_longitude_radians;
  /**
   * Altitude of centre poing of MWA in metres
   */
  double mwa_altitude_metres;
  /**
   * the velocity factor of electic fields in RG-6 like coax
   */
  double coax_v_factor;
  /**
   * ATTEN_DB  // global analogue attenuation, in dB
   */
  double global_analogue_attenuation_db;
  /**
   * RA tile pointing
   */
  double ra_tile_pointing_degrees;
  /**
   * DEC tile pointing
   */
  double dec_tile_pointing_degrees;
  /**
   * RA phase centre
   */
  double ra_phase_center_degrees;
  /**
   * DEC phase centre
   */
  double dec_phase_center_degrees;
  /**
   * AZIMUTH
   */
  double azimuth_degrees;
  /**
   * ALTITUDE
   */
  double altitude_degrees;
  /**
   * Altitude of Sun
   */
  double sun_altitude_degrees;
  /**
   * Distance from pointing center to Sun
   */
  double sun_distance_degrees;
  /**
   * Distance from pointing center to the Moon
   */
  double moon_distance_degrees;
  /**
   * Distance from pointing center to Jupiter
   */
  double jupiter_distance_degrees;
  /**
   * Local Sidereal Time
   */
  double lst_degrees;
  /**
   * Hour Angle of pointing center (as a string)
   */
  char *hour_angle_string;
  /**
   * GRIDNAME
   */
  char *grid_name;
  /**
   * GRIDNUM
   */
  int32_t grid_number;
  /**
   * CREATOR
   */
  char *creator;
  /**
   * PROJECT
   */
  char *project_id;
  /**
   * Observation name
   */
  char *observation_name;
  /**
   * MWA observation mode
   */
  char *mode;
  /**
   * Scheduled start (gps time) of observation
   */
  int64_t scheduled_start_utc;
  /**
   * Scheduled end (gps time) of observation
   */
  int64_t scheduled_end_utc;
  /**
   * Scheduled start (MJD) of observation
   */
  double scheduled_start_mjd;
  /**
   * Scheduled end (MJD) of observation
   */
  double scheduled_end_mjd;
  /**
   * Scheduled start (UNIX time) of observation
   */
  uint64_t scheduled_start_unix_time_milliseconds;
  /**
   * Scheduled end (UNIX time) of observation
   */
  uint64_t scheduled_end_unix_time_milliseconds;
  /**
   * Scheduled duration of observation
   */
  uint64_t scheduled_duration_milliseconds;
  /**
   * Seconds of bad data after observation starts
   */
  uint64_t quack_time_duration_milliseconds;
  /**
   * OBSID+QUACKTIM as Unix timestamp (first good timestep)
   */
  uint64_t good_time_unix_milliseconds;
  /**
   * Total number of antennas (tiles) in the array
   */
  uintptr_t num_antennas;
  /**
   * The Metafits defines an rf chain for antennas(tiles) * pol(X,Y)
   */
  uintptr_t num_rf_inputs;
  /**
   * Number of antenna pols. e.g. X and Y
   */
  uintptr_t num_antenna_pols;
  /**
   * Number of coarse channels
   */
  uintptr_t num_coarse_channels;
  /**
   * Total bandwidth of observation (of the coarse channels we have)
   */
  uint32_t observation_bandwidth_hz;
  /**
   * Bandwidth of each coarse channel
   */
  uint32_t coarse_channel_width_hz;
} mwalibMetafitsMetadata;

/**
 *
 * C Representation of the `CorrelatorContext` metadata
 *
 */
typedef struct {
  /**
   * Version of the correlator format
   */
  CorrelatorVersion corr_version;
  /**
   * The proper start of the observation (the time that is common to all
   * provided gpubox files).
   */
  uint64_t start_unix_time_milliseconds;
  /**
   * `end_time_milliseconds` will is the actual end time of the observation
   * i.e. start time of last common timestep plus integration time.
   */
  uint64_t end_unix_time_milliseconds;
  /**
   * Total duration of observation (based on gpubox files)
   */
  uint64_t duration_milliseconds;
  /**
   * Number of timesteps in the observation
   */
  uintptr_t num_timesteps;
  /**
   * Number of baselines stored. This is autos plus cross correlations
   */
  uintptr_t num_baselines;
  /**
   * Number of polarisation combinations in the visibilities e.g. XX,XY,YX,YY == 4
   */
  uintptr_t num_visibility_pols;
  /**
   * Correlator mode dump time
   */
  uint64_t integration_time_milliseconds;
  /**
   * Number of coarse channels
   */
  uintptr_t num_coarse_channels;
  /**
   * Total bandwidth of observation (of the coarse channels we have)
   */
  uint32_t bandwidth_hz;
  /**
   * Bandwidth of each coarse channel
   */
  uint32_t coarse_channel_width_hz;
  /**
   * Correlator fine_channel_resolution
   */
  uint32_t fine_channel_width_hz;
  /**
   * Number of fine channels in each coarse channel
   */
  uintptr_t num_fine_channels_per_coarse;
  /**
   * The number of bytes taken up by a scan/timestep in each gpubox file.
   */
  uintptr_t num_timestep_coarse_channel_bytes;
  /**
   * The number of floats in each gpubox HDU.
   */
  uintptr_t num_timestep_coarse_channel_floats;
  /**
   * This is the number of gpubox files *per batch*.
   */
  uintptr_t num_gpubox_files;
} mwalibCorrelatorMetadata;

/**
 *
 * C Representation of the `VoltageContext` metadata
 *
 */
typedef struct {
  /**
   * Version of the correlator format
   */
  CorrelatorVersion corr_version;
  /**
   * The proper start of the observation (the time that is common to all
   * provided voltage files).
   */
  uint64_t start_gps_time_milliseconds;
  /**
   * `end_gps_time_milliseconds` is the actual end time of the observation
   * i.e. start time of last common timestep plus length of a voltage file (1 sec for MWA Legacy, 8 secs for MWAX).
   */
  uint64_t end_gps_time_milliseconds;
  /**
   * Total duration of observation (based on voltage files)
   */
  uint64_t duration_milliseconds;
  /**
   * Number of timesteps in the observation
   */
  uintptr_t num_timesteps;
  /**
   * Number of coarse channels after we've validated the input voltage files
   */
  uintptr_t num_coarse_channels;
  /**
   * Total bandwidth of observation (of the coarse channels we have)
   */
  uint32_t bandwidth_hz;
  /**
   * Bandwidth of each coarse channel
   */
  uint32_t coarse_channel_width_hz;
  /**
   * Volatge fine_channel_resolution (if applicable- MWA legacy is 10 kHz, MWAX is unchannelised i.e. the full coarse channel width)
   */
  uint32_t fine_channel_width_hz;
  /**
   * Number of fine channels in each coarse channel
   */
  uintptr_t num_fine_channels_per_coarse;
} mwalibVoltageMetadata;

/**
 * Representation in C of an `mwalibAntenna` struct
 */
typedef struct {
  /**
   * This is the antenna number.
   * Nominally this is the field we sort by to get the desired output order of antenna.
   * X and Y have the same antenna number. This is the sorted ordinal order of the antenna.None
   * e.g. 0...N-1
   */
  uint32_t antenna;
  /**
   * Numeric part of tile_name for the antenna. Each pol has the same value
   * e.g. tile_name "tile011" hsa tile_id of 11
   */
  uint32_t tile_id;
  /**
   * Human readable name of the antenna
   * X and Y have the same name
   */
  char *tile_name;
} mwalibAntenna;

/**
 *
 * C Representation of a `mwalibBaseline` struct
 *
 */
typedef struct {
  /**
   * Index in the `MetafitsContext` antenna array for antenna1 for this baseline
   */
  uintptr_t antenna1_index;
  /**
   * Index in the `MetafitsContext` antenna array for antenna2 for this baseline
   */
  uintptr_t antenna2_index;
} mwalibBaseline;

/**
 * Representation in C of an `mwalibCoarseChannel` struct
 */
typedef struct {
  /**
   * Correlator channel is 0 indexed (0..N-1)
   */
  uintptr_t correlator_channel_number;
  /**
   * Receiver channel is 0-255 in the RRI recivers
   */
  uintptr_t receiver_channel_number;
  /**
   * gpubox channel number
   * Legacy e.g. obsid_datetime_gpuboxXX_00
   * v2     e.g. obsid_datetime_gpuboxXXX_00
   */
  uintptr_t gpubox_number;
  /**
   * Width of a coarse channel in Hz
   */
  uint32_t channel_width_hz;
  /**
   * Starting frequency of coarse channel in Hz
   */
  uint32_t channel_start_hz;
  /**
   * Centre frequency of coarse channel in Hz
   */
  uint32_t channel_centre_hz;
  /**
   * Ending frequency of coarse channel in Hz
   */
  uint32_t channel_end_hz;
} mwalibCoarseChannel;

/**
 * Representation in C of an `mwalibRFInput` struct
 */
typedef struct {
  /**
   * This is the metafits order (0-n inputs)
   */
  uint32_t input;
  /**
   * This is the antenna number.
   * Nominally this is the field we sort by to get the desired output order of antenna.
   * X and Y have the same antenna number. This is the sorted ordinal order of the antenna.None
   * e.g. 0...N-1
   */
  uint32_t antenna;
  /**
   * Numeric part of tile_name for the antenna. Each pol has the same value
   * e.g. tile_name "tile011" hsa tile_id of 11
   */
  uint32_t tile_id;
  /**
   * Human readable name of the antenna
   * X and Y have the same name
   */
  char *tile_name;
  /**
   * Polarisation - X or Y
   */
  char *pol;
  /**
   * Electrical length in metres for this antenna and polarisation to the receiver
   */
  double electrical_length_m;
  /**
   * Antenna position North from the array centre (metres)
   */
  double north_m;
  /**
   * Antenna position East from the array centre (metres)
   */
  double east_m;
  /**
   * Antenna height from the array centre (metres)
   */
  double height_m;
  /**
   * AKA PFB to correlator input order (only relevant for pre V2 correlator)
   */
  uint32_t vcs_order;
  /**
   * Subfile order is the order in which this rf_input is desired in our final output of data
   */
  uint32_t subfile_order;
  /**
   * Is this rf_input flagged out (due to tile error, etc from metafits)
   */
  bool flagged;
  /**
   * Receiver number
   */
  uint32_t receiver_number;
  /**
   * Receiver slot number
   */
  uint32_t receiver_slot_number;
} mwalibRFInput;

/**
 *
 * C Representation of a `mwalibTimeStep` struct
 *
 */
typedef struct {
  /**
   * UNIX time (in milliseconds to avoid floating point inaccuracy)
   */
  uint64_t unix_time_milliseconds;
  uint64_t gps_time_milliseconds;
} mwalibTimeStep;

/**
 *
 * C Representation of a `mwalibVisibilityPol` struct
 *
 */
typedef struct {
  /**
   * Polarisation (e.g. "XX" or "XY" or "YX" or "YY")
   */
  char *polarisation;
} mwalibVisibilityPol;

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
 * * 0 on success, non-zero on failure
 *
 * # Safety
 * * rust_cstring must not have already been freed and must point to a Rust string.
 */
int32_t mwalib_free_rust_cstring(char *rust_cstring);

/**
 * Create and return a pointer to an `MetafitsContext` struct given only a metafits file
 *
 * # Arguments
 *
 * * `metafits_filename` - pointer to char* buffer containing the full path and filename of a metafits file.
 *
 * * `out_metafits_context_ptr` - A Rust-owned populated `MetafitsContext` pointer. Free with `mwalib_metafits_context_free'.
 *
 * * `error_message` - pointer to already allocated buffer for any error messages to be returned to the caller.
 *
 * * `error_message_length` - length of error_message char* buffer.
 *
 *
 * # Returns
 *
 * * 0 on success, non-zero on failure
 *
 *
 * # Safety
 * * `error_message` *must* point to an already allocated `char*` buffer for any error messages.
 * * Caller *must* call the `mwalib_metafits_context_free` function to release the rust memory.
 */
int32_t mwalib_metafits_context_new(const char *metafits_filename,
                                    MetafitsContext **out_metafits_context_ptr,
                                    const char *error_message,
                                    size_t error_message_length);

/**
 * Display an `MetafitsContext` struct.
 *
 *
 * # Arguments
 *
 * * `metafits_context_ptr` - pointer to an already populated `MetafitsContext` object
 *
 * * `error_message` - pointer to already allocated buffer for any error messages to be returned to the caller.
 *
 * * `error_message_length` - length of error_message char* buffer.
 *
 *
 * # Returns
 *
 * * 0 on success, non-zero on failure
 *
 *
 * # Safety
 * * `error_message` *must* point to an already allocated char* buffer for any error messages.
 * * `metafits_context_ptr` must contain an MetafitsContext object already populated via `mwalib_metafits_context_new`
 */
int32_t mwalib_metafits_context_display(const MetafitsContext *metafits_context_ptr,
                                        const char *error_message,
                                        size_t error_message_length);

/**
 * Free a previously-allocated `MetafitsContext` struct (and it's members).
 *
 * # Arguments
 *
 * * `metafits_context_ptr` - pointer to an already populated `MetafitsContext` object
 *
 *
 * # Returns
 *
 * * 0 on success, non-zero on failure
 *
 *
 * # Safety
 * * This must be called once caller is finished with the `MetafitsContext` object
 * * `metafits_context_ptr` must point to a populated `MetafitsContext` object from the `mwalib_metafits_context_new` functions.
 * * `metafits_context_ptr` must not have already been freed.
 */
int32_t mwalib_metafits_context_free(MetafitsContext *metafits_context_ptr);

/**
 * Create and return a pointer to an `CorrelatorContext` struct based on metafits and gpubox files
 *
 * # Arguments
 *
 * * `metafits_filename` - pointer to char* buffer containing the full path and filename of a metafits file.
 *
 * * `gpubox_filenames` - pointer to array of char* buffers containing the full path and filename of the gpubox FITS files.
 *
 * * `gpubox_count` - length of the gpubox char* array.
 *
 * * `out_correlator_context_ptr` - A Rust-owned populated `CorrelatorContext` pointer. Free with `mwalib_correlator_context_free`.
 *
 * * `error_message` - pointer to already allocated buffer for any error messages to be returned to the caller.
 *
 * * `error_message_length` - length of error_message char* buffer.
 *
 *
 * # Returns
 *
 * * 0 on success, non-zero on failure
 *
 *
 * # Safety
 * * `error_message` *must* point to an already allocated `char*` buffer for any error messages.
 * * Caller *must* call function `mwalib_correlator_context_free` to release the rust memory.
 */
int32_t mwalib_correlator_context_new(const char *metafits_filename,
                                      const char **gpubox_filenames,
                                      size_t gpubox_count,
                                      CorrelatorContext **out_correlator_context_ptr,
                                      const char *error_message,
                                      size_t error_message_length);

/**
 * Display an `CorrelatorContext` struct.
 *
 *
 * # Arguments
 *
 * * `correlator_context_ptr` - pointer to an already populated `CorrelatorContext` object
 *
 * * `error_message` - pointer to already allocated buffer for any error messages to be returned to the caller.
 *
 * * `error_message_length` - length of error_message char* buffer.
 *
 *
 * # Returns
 *
 * * 0 on success, non-zero on failure
 *
 *
 * # Safety
 * * `error_message` *must* point to an already allocated char* buffer for any error messages.
 * * `correlator_context_ptr` must contain an `CorrelatorContext` object already populated via `mwalib_correlator_context_new`
 */
int32_t mwalib_correlator_context_display(const CorrelatorContext *correlator_context_ptr,
                                          const char *error_message,
                                          size_t error_message_length);

/**
 * Read a single timestep / coarse channel of MWA data.
 *
 * This method takes as input a timestep_index and a coarse_channel_index to return one
 * HDU of data in [baseline][freq][pol][r][i] format
 *
 * # Arguments
 *
 * * `correlator_context_ptr` - pointer to an already populated `CorrelatorContext` object.
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
 * * 0 on success, non-zero on failure
 *
 *
 * # Safety
 * * `error_message` *must* point to an already allocated char* buffer for any error messages.
 * * `correlator_context_ptr` must point to a populated object from the `mwalib_correlator_context_new` function.
 * * Caller *must* call `mwalib_correlator_context_free_read_buffer` function to release the rust memory.
 */
int32_t mwalib_correlator_context_read_by_baseline(CorrelatorContext *correlator_context_ptr,
                                                   uintptr_t timestep_index,
                                                   uintptr_t coarse_channel_index,
                                                   float *buffer_ptr,
                                                   size_t buffer_len,
                                                   const char *error_message,
                                                   size_t error_message_length);

/**
 * Read a single timestep / coarse channel of MWA data.
 *
 * This method takes as input a timestep_index and a coarse_channel_index to return one
 * HDU of data in [freq][baseline][pol][r][i] format
 *
 * # Arguments
 *
 * * `correlator_context_ptr` - pointer to an already populated `CorrelatorContext` object.
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
 * * 0 on success, non-zero on failure
 *
 *
 * # Safety
 * * `error_message` *must* point to an already allocated char* buffer for any error messages.
 * * `correlator_context_ptr` must point to a populated object from the `mwalib_correlator_context_new` function.
 * * Caller *must* call `mwalib_correlator_context_free_read_buffer` function to release the rust memory.
 */
int32_t mwalib_correlator_context_read_by_frequency(CorrelatorContext *correlator_context_ptr,
                                                    uintptr_t timestep_index,
                                                    uintptr_t coarse_channel_index,
                                                    float *buffer_ptr,
                                                    size_t buffer_len,
                                                    const char *error_message,
                                                    size_t error_message_length);

/**
 * Free a previously-allocated `CorrelatorContext` struct (and it's members).
 *
 * # Arguments
 *
 * * `correlator_context_ptr` - pointer to an already populated `CorrelatorContext` object
 *
 *
 * # Returns
 *
 * * 0 on success, non-zero on failure
 *
 *
 * # Safety
 * * This must be called once caller is finished with the `CorrelatorContext` object
 * * `correlator_context_ptr` must point to a populated `CorrelatorContext` object from the `mwalib_correlator_context_new` function.
 * * `correlator_context_ptr` must not have already been freed.
 */
int32_t mwalib_correlator_context_free(CorrelatorContext *correlator_context_ptr);

/**
 * Create and return a pointer to an `VoltageContext` struct based on metafits and voltage files
 *
 * # Arguments
 *
 * * `metafits_filename` - pointer to char* buffer containing the full path and filename of a metafits file.
 *
 * * `voltage_filenames` - pointer to array of char* buffers containing the full path and filename of the voltage files.
 *
 * * `voltage_file_count` - length of the voltage char* array.
 *
 * * `out_voltage_context_ptr` - A Rust-owned populated `VoltageContext` pointer. Free with `mwalib_voltage_context_free`.
 *
 * * `error_message` - pointer to already allocated buffer for any error messages to be returned to the caller.
 *
 * * `error_message_length` - length of error_message char* buffer.
 *
 *
 * # Returns
 *
 * * 0 on success, non-zero on failure
 *
 *
 * # Safety
 * * `error_message` *must* point to an already allocated `char*` buffer for any error messages.
 * * Caller *must* call function `mwalib_voltage_context_free` to release the rust memory.
 */
int32_t mwalib_voltage_context_new(const char *metafits_filename,
                                   const char **voltage_filenames,
                                   size_t voltage_file_count,
                                   VoltageContext **out_voltage_context_ptr,
                                   const char *error_message,
                                   size_t error_message_length);

/**
 * Display a `VoltageContext` struct.
 *
 *
 * # Arguments
 *
 * * `voltage_context_ptr` - pointer to an already populated `VoltageContext` object
 *
 * * `error_message` - pointer to already allocated buffer for any error messages to be returned to the caller.
 *
 * * `error_message_length` - length of error_message char* buffer.
 *
 *
 * # Returns
 *
 * * 0 on success, non-zero on failure
 *
 *
 * # Safety
 * * `error_message` *must* point to an already allocated char* buffer for any error messages.
 * * `voltage_context_ptr` must contain an `VoltageContext` object already populated via `mwalib_voltage_context_new`
 */
int32_t mwalib_voltage_context_display(const VoltageContext *voltage_context_ptr,
                                       const char *error_message,
                                       size_t error_message_length);

/**
 * Free a previously-allocated `VoltageContext` struct (and it's members).
 *
 * # Arguments
 *
 * * `voltage_context_ptr` - pointer to an already populated `VoltageContext` object
 *
 *
 * # Returns
 *
 * * 0 on success, non-zero on failure
 *
 *
 * # Safety
 * * This must be called once caller is finished with the `VoltageContext` object
 * * `voltage_context_ptr` must point to a populated `VoltageContext` object from the `mwalib_voltage_context_new` function.
 * * `voltage_context_ptr` must not have already been freed.
 */
int32_t mwalib_voltage_context_free(VoltageContext *voltage_context_ptr);

/**
 * This passed back a struct containing the `MetafitsContext` metadata, given a MetafitsContext, CorrelatorContext or VoltageContext
 *
 * # Arguments
 *
 * * `metafits_context_ptr` - pointer to an already populated `MetafitsContext` object. (Exclusive with correlator_context_ptr and voltage_context_ptr)
 *
 * * `correlator_context_ptr` - pointer to an already populated `CorrelatorContext` object. (Exclusive with metafits_context_ptr and voltage_context_ptr)
 *
 * * `voltage_context_ptr` - pointer to an already populated `VoltageContext` object. (Exclusive with metafits_context_ptr and correlator_context_ptr)
 *
 * * `out_metafits_metadata_ptr` - pointer to a Rust-owned `mwalibMetafitsMetadata` struct. Free with `mwalib_metafits_metadata_free`
 *
 * * `error_message` - pointer to already allocated buffer for any error messages to be returned to the caller.
 *
 * * `error_message_length` - length of error_message char* buffer.
 *
 *
 * # Returns
 *
 * * 0 on success, non-zero on failure
 *
 *
 * # Safety
 * * `error_message` *must* point to an already allocated char* buffer for any error messages.
 * * `metafits_context_ptr` must point to a populated MetafitsContext object from the `mwalib_metafits_context_new` function OR
 * * `correlator_context_ptr` must point to a populated CorrelatorContext object from the 'mwalib_correlator_context_new' function OR
 * * `voltage_context_ptr` must point to a populated VoltageContext object from the `mwalib_voltage_context_new` function. (Set the unused contexts to NULL).
 * * Caller must call `mwalib_metafits_metadata_free` once finished, to free the rust memory.
 */
int32_t mwalib_metafits_metadata_get(MetafitsContext *metafits_context_ptr,
                                     CorrelatorContext *correlator_context_ptr,
                                     VoltageContext *voltage_context_ptr,
                                     mwalibMetafitsMetadata **out_metafits_metadata_ptr,
                                     const char *error_message,
                                     size_t error_message_length);

/**
 * Free a previously-allocated `mwalibMetafitsMetadata` struct.
 *
 * # Arguments
 *
 * * `metafits_metadata_ptr` - pointer to an already populated `mwalibMetafitsMetadata` object
 *
 *
 * # Returns
 *
 * * 0 on success, non-zero on failure
 *
 *
 * # Safety
 * * This must be called once caller is finished with the `mwalibMetafitsMetadata` object
 * * `metafits_metadata_ptr` must point to a populated `mwalibMetafitsMetadata` object from the `mwalib_metafits_metadata_get` function.
 * * `metafits_metadata_ptr` must not have already been freed.
 */
int32_t mwalib_metafits_metadata_free(mwalibMetafitsMetadata *metafits_metadata_ptr);

/**
 * This returns a struct containing the `CorrelatorContext` metadata
 *
 * # Arguments
 *
 * * `correlator_context_ptr` - pointer to an already populated `CorrelatorContext` object.
 *
 * * `out_correaltor_metadata_ptr` - A Rust-owned populated `mwalibCorrelatorMetadata` struct. Free with `mwalib_correlator_metadata_free`.
 *
 * * `error_message` - pointer to already allocated buffer for any error messages to be returned to the caller.
 *
 * * `error_message_length` - length of error_message char* buffer.
 *
 *
 * # Returns
 *
 * * 0 on success, non-zero on failure
 *
 *
 * # Safety
 * * `error_message` *must* point to an already allocated char* buffer for any error messages.
 * * `correlator_context_ptr` must point to a populated `CorrelatorContext` object from the `mwalib_correlator_context_new` function.
 * * Caller must call `mwalib_correlator_metadata_free` once finished, to free the rust memory.
 */
int32_t mwalib_correlator_metadata_get(CorrelatorContext *correlator_context_ptr,
                                       mwalibCorrelatorMetadata **out_correlator_metadata_ptr,
                                       const char *error_message,
                                       size_t error_message_length);

/**
 * Free a previously-allocated `mwalibCorrelatorMetadata` struct.
 *
 * # Arguments
 *
 * * `correlator_metadata_ptr` - pointer to an already populated `mwalibCorrelatorMetadata` object
 *
 *
 * # Returns
 *
 * * 0 on success, non-zero on failure
 *
 *
 * # Safety
 * * This must be called once caller is finished with the `mwalibCorrelatorMetadata` object
 * * `correlator_metadata_ptr` must point to a populated `mwalibCorrelatorMetadata` object from the `mwalib_correlator_metadata_get` function.
 * * `correlator_metadata_ptr` must not have already been freed.
 */
int32_t mwalib_correlator_metadata_free(mwalibCorrelatorMetadata *correlator_metadata_ptr);

/**
 * This returns a struct containing the `VoltageContext` metadata
 *
 * # Arguments
 *
 * * `voltage_context_ptr` - pointer to an already populated `VoltageContext` object.
 *
 * * `out_voltage_metadata_ptr` - A Rust-owned populated `mwalibCorrelatorMetadata` struct. Free with `mwalib_correlator_metadata_free`.
 *
 * * `error_message` - pointer to already allocated buffer for any error messages to be returned to the caller.
 *
 * * `error_message_length` - length of error_message char* buffer.
 *
 *
 * # Returns
 *
 * * 0 on success, non-zero on failure
 *
 *
 * # Safety
 * * `error_message` *must* point to an already allocated char* buffer for any error messages.
 * * `correlator_context_ptr` must point to a populated `VoltageContext` object from the `mwalib_correlator_context_new` function.
 * * Caller must call `mwalib_correlator_metadata_free` once finished, to free the rust memory.
 */
int32_t mwalib_voltage_metadata_get(VoltageContext *voltage_context_ptr,
                                    mwalibVoltageMetadata **out_voltage_metadata_ptr,
                                    const char *error_message,
                                    size_t error_message_length);

/**
 * Free a previously-allocated `mwalibVoltageMetadata` struct.
 *
 * # Arguments
 *
 * * `voltage_metadata_ptr` - pointer to an already populated `mwalibVoltageMetadata` object
 *
 *
 * # Returns
 *
 * * 0 on success, non-zero on failure
 *
 *
 * # Safety
 * * This must be called once caller is finished with the `mwalibVoltageMetadata` object
 * * `voltage_metadata_ptr` must point to a populated `mwalibVoltageMetadata` object from the `mwalib_voltage_metadata_get` function.
 * * `voltage_metadata_ptr` must not have already been freed.
 */
int32_t mwalib_voltage_metadata_free(mwalibVoltageMetadata *voltage_metadata_ptr);

/**
 * This passes back an array of structs containing all antennas given a metafits OR correlator context.
 *
 * # Arguments
 *
 * * `metafits_context_ptr` - pointer to an already populated `MetafitsContext` object. (Exclusive with `correlator_context_ptr` and `voltage_context_ptr`)
 *
 * * `correlator_context_ptr` - pointer to an already populated `CorrelatorContext` object. (Exclusive with `metafits_context_ptr` and `voltage_context_ptr`)
 *
 * * `voltage_context_ptr` - pointer to an already populated `VoltageContext` object. (Exclusive with `metafits_context_ptr` and `correlator_context_ptr`)
 *
 * * `out_antennas_ptr` - A Rust-owned populated array of `mwalibAntenna` struct. Free with `mwalib_antennas_free`.
 *
 * * `out_antennas_len` - Antennas array length.
 *
 * * `error_message` - pointer to already allocated buffer for any error messages to be returned to the caller.
 *
 * * `error_message_length` - length of error_message char* buffer.
 *
 *
 * # Returns
 *
 * * 0 on success, non-zero on failure
 *
 *
 * # Safety
 * * `error_message` *must* point to an already allocated char* buffer for any error messages.
 * * `metafits_context_ptr` must point to a populated MetafitsContext object from the `mwalib_metafits_context_new` function.
 * * Caller must call `mwalib_antenna_free` once finished, to free the rust memory.
 */
int32_t mwalib_antennas_get(MetafitsContext *metafits_context_ptr,
                            CorrelatorContext *correlator_context_ptr,
                            VoltageContext *voltage_context_ptr,
                            mwalibAntenna **out_antennas_ptr,
                            uintptr_t *out_antennas_len,
                            const char *error_message,
                            size_t error_message_length);

/**
 * Free a previously-allocated `mwalibAntenna` array of structs.
 *
 * # Arguments
 *
 * * `antennas_ptr` - pointer to an already populated `mwalibAntenna` array
 *
 * * `antennas_len` - number of elements in the pointed to array
 *
 *
 * # Returns
 *
 * * 0 on success, non-zero on failure
 *
 *
 * # Safety
 * * This must be called once caller is finished with the `mwalibAntenna` array
 * * `antenna_ptr` must point to a populated `mwalibAntenna` array from the `mwalib_antennas_get` function.
 * * `antenna_ptr` must not have already been freed.
 */
int32_t mwalib_antennas_free(mwalibAntenna *antennas_ptr,
                             uintptr_t antennas_len);

/**
 * This passes a pointer to an array of baselines
 *
 * # Arguments
 *
 * * `correlator_context_ptr` - pointer to an already populated `CorrelatorContext` object.
 *
 * * `out_baselines_ptr` - populated, array of rust-owned baseline structs. Free with `mwalib_baselines_free`.
 *
 * * `out_baselines_len` - baseline array length.
 *
 * * `error_message` - pointer to already allocated buffer for any error messages to be returned to the caller.
 *
 * * `error_message_length` - length of error_message char* buffer.
 *
 *
 * # Returns
 *
 * 0 on success, non-zero on failure
 *
 *
 * # Safety
 * * `error_message` *must* point to an already allocated char* buffer for any error messages.
 * * `correlator_context_ptr` must point to a populated `CorrelatorContext` object from the `mwalib_correlator_context_new` function.
 * * Caller must call `mwalib_baselines_free` once finished, to free the rust memory.
 */
int32_t mwalib_correlator_baselines_get(CorrelatorContext *correlator_context_ptr,
                                        mwalibBaseline **out_baselines_ptr,
                                        uintptr_t *out_baselines_len,
                                        const char *error_message,
                                        size_t error_message_length);

/**
 * Free a previously-allocated `mwalibBaseline` struct.
 *
 * # Arguments
 *
 * * `baselines_ptr` - pointer to an already populated `mwalibBaseline` array
 *
 * * `baselines_len` - number of elements in the pointed to array
 *
 *
 * # Returns
 *
 * * 0 on success, non-zero on failure
 *
 *
 * # Safety
 * * This must be called once caller is finished with the `mwalibBaseline` array
 * * `baseline_ptr` must point to a populated `mwalibBaseline` array from the `mwalib_baselines_get` function.
 * * `baseline_ptr` must not have already been freed.
 */
int32_t mwalib_baselines_free(mwalibBaseline *baselines_ptr,
                              uintptr_t baselines_len);

/**
 * This passes a pointer to an array of correlator coarse channel
 *
 * # Arguments
 *
 * * `correlator_context_ptr` - pointer to an already populated `CorrelatorContext` object.
 *
 * * `out_coarse_channels_ptr` - A Rust-owned populated `mwalibCoarseChannel` array of structs. Free with `mwalib_coarse_channels_free`.
 *
 * * `out_coarse_channels_len` - Coarse channel array length.
 *
 * * `error_message` - pointer to already allocated buffer for any error messages to be returned to the caller.
 *
 * * `error_message_length` - length of error_message char* buffer.
 *
 *
 * # Returns
 *
 * * 0 on success, non-zero on failure
 *
 *
 * # Safety
 * * `error_message` *must* point to an already allocated char* buffer for any error messages.
 * * `correlator_context_ptr` must point to a populated `mwalibCorrelatorContext` object from the `mwalib_correlator_context_new` function.
 * * Caller must call `mwalib_coarse_channels_free` once finished, to free the rust memory.
 */
int32_t mwalib_correlator_coarse_channels_get(CorrelatorContext *correlator_context_ptr,
                                              mwalibCoarseChannel **out_coarse_channels_ptr,
                                              uintptr_t *out_coarse_channels_len,
                                              const char *error_message,
                                              size_t error_message_length);

/**
 * This passes a pointer to an array of voltage coarse channel
 *
 * # Arguments
 *
 * * `voltage_context_ptr` - pointer to an already populated `VoltageContext` object.
 *
 * * `out_coarse_channels_ptr` - A Rust-owned populated `mwalibCoarseChannel` array of structs. Free with `mwalib_coarse_channels_free`.
 *
 * * `out_coarse_channels_len` - Coarse channel array length.
 *
 * * `error_message` - pointer to already allocated buffer for any error messages to be returned to the caller.
 *
 * * `error_message_length` - length of error_message char* buffer.
 *
 *
 * # Returns
 *
 * * 0 on success, non-zero on failure
 *
 *
 * # Safety
 * * `error_message` *must* point to an already allocated char* buffer for any error messages.
 * * `voltage_context_ptr` must point to a populated `mwalibVoltageContext` object from the `mwalib_voltage_context_new` function.
 * * Caller must call `mwalib_coarse_channels_free` once finished, to free the rust memory.
 */
int32_t mwalib_voltage_coarse_channels_get(VoltageContext *voltage_context_ptr,
                                           mwalibCoarseChannel **out_coarse_channels_ptr,
                                           uintptr_t *out_coarse_channels_len,
                                           const char *error_message,
                                           size_t error_message_length);

/**
 * Free a previously-allocated `mwalibCoarseChannel` struct.
 *
 * # Arguments
 *
 * * `coarse_channels_ptr` - pointer to an already populated `mwalibCoarseChannel` array
 *
 * * `coarse_channels_len` - number of elements in the pointed to array
 *
 *
 * # Returns
 *
 * * 0 on success, non-zero on failure
 *
 *
 * # Safety
 * * This must be called once caller is finished with the `mwalibCoarseChannel` array
 * * `coarse_channel_ptr` must point to a populated `mwalibCoarseChannel` array from the `mwalib_correlator_coarse_channels_get` function.
 * * `coarse_channel_ptr` must not have already been freed.
 */
int32_t mwalib_coarse_channels_free(mwalibCoarseChannel *coarse_channels_ptr,
                                    uintptr_t coarse_channels_len);

/**
 * This passes a pointer to an array of antenna given a metafits context OR correlator context
 *
 * # Arguments
 *
 * * `metafits_context_ptr` - pointer to an already populated `MetafitsContext` object. (Exclusive with `correlator_context_ptr` and `voltage_context_ptr`)
 *
 * * `correlator_context_ptr` - pointer to an already populated `CorrelatorContext` object. (Exclusive with `metafits_context_ptr` and `voltage_context_ptr`)
 *
 * * `voltage_context_ptr` - pointer to an already populated `VoltageContext` object. (Exclusive with `metafits_context_ptr` and `correlator_context_ptr`)
 *
 * * `out_rfinputs_ptr` - A Rust-owned populated `mwalibRFInput` array of structs. Free with `mwalib_rfinputs_free`.
 *
 * * `out_rfinputs_len` - rfinputs array length.
 *
 * * `error_message` - pointer to already allocated buffer for any error messages to be returned to the caller.
 *
 * * `error_message_length` - length of error_message char* buffer.
 *
 *
 * # Returns
 *
 * * 0 on success, non-zero on failure
 *
 *
 * # Safety
 * * `error_message` *must* point to an already allocated char* buffer for any error messages.
 * * `metafits_context_ptr` must point to a populated `MetafitsContext` object from the `mwalib_metafits_context_new` function.
 * * Caller must call `mwalib_rfinputs_free` once finished, to free the rust memory.
 */
int32_t mwalib_rfinputs_get(MetafitsContext *metafits_context_ptr,
                            CorrelatorContext *correlator_context_ptr,
                            VoltageContext *voltage_context_ptr,
                            mwalibRFInput **out_rfinputs_ptr,
                            uintptr_t *out_rfinputs_len,
                            const char *error_message,
                            size_t error_message_length);

/**
 * Free a previously-allocated `mwalibRFInput` struct.
 *
 * # Arguments
 *
 * * `rf_inputs_ptr` - pointer to an already populated `mwalibRFInput` object
 *
 * * `rf_inputs_len` - number of elements in the pointed to array
 *
 *
 * # Returns
 *
 * * 0 on success, non-zero on failure
 *
 *
 * # Safety
 * * This must be called once caller is finished with the `mwalibRFInput` array
 * * `rf_input_ptr` must point to a populated `mwalibRFInput` array from the `mwalib_rfinputs_get` function.
 * * `rf_input_ptr` must not have already been freed.
 */
int32_t mwalib_rfinputs_free(mwalibRFInput *rf_inputs_ptr,
                             uintptr_t rf_inputs_len);

/**
 * This passes a pointer to an array of timesteps
 *
 * # Arguments
 *
 * * `correlator_context_ptr` - pointer to an already populated `CorrelatorContext` object.
 *
 * * `out_timesteps_ptr` - A Rust-owned populated `mwalibTimeStep` struct. Free with `mwalib_timestep_free`.
 *
 * * `out_timesteps_len` - Timesteps array length.
 *
 * * `error_message` - pointer to already allocated buffer for any error messages to be returned to the caller.
 *
 * * `error_message_length` - length of error_message char* buffer.
 *
 *
 * # Returns
 *
 * * 0 on success, non-zero on failure
 *
 *
 * # Safety
 * * `error_message` *must* point to an already allocated char* buffer for any error messages.
 * * `correlator_context_ptr` must point to a populated `CorrelatorContext` object from the `mwalib_correlator_context_new` function.
 * * Caller must call `mwalib_timestep_free` once finished, to free the rust memory.
 */
int32_t mwalib_correlator_timesteps_get(CorrelatorContext *correlator_context_ptr,
                                        mwalibTimeStep **out_timesteps_ptr,
                                        uintptr_t *out_timesteps_pols_len,
                                        const char *error_message,
                                        size_t error_message_length);

/**
 * This passes a pointer to an array of timesteps
 *
 * # Arguments
 *
 * * `voltage_context_ptr` - pointer to an already populated `VoltageContext` object.
 *
 * * `out_timesteps_ptr` - A Rust-owned populated `mwalibTimeStep` struct. Free with `mwalib_timestep_free`.
 *
 * * `out_timesteps_len` - Timesteps array length.
 *
 * * `error_message` - pointer to already allocated buffer for any error messages to be returned to the caller.
 *
 * * `error_message_length` - length of error_message char* buffer.
 *
 *
 * # Returns
 *
 * * 0 on success, non-zero on failure
 *
 *
 * # Safety
 * * `error_message` *must* point to an already allocated char* buffer for any error messages.
 * * `voltage_context_ptr` must point to a populated `VoltageContext` object from the `mwalib_voltage_context_new` function.
 * * Caller must call `mwalib_timestep_free` once finished, to free the rust memory.
 */
int32_t mwalib_voltage_timesteps_get(VoltageContext *voltage_context_ptr,
                                     mwalibTimeStep **out_timesteps_ptr,
                                     uintptr_t *out_timesteps_pols_len,
                                     const char *error_message,
                                     size_t error_message_length);

/**
 * Free a previously-allocated `mwalibTimeStep` struct.
 *
 * # Arguments
 *
 * * `timesteps_ptr` - pointer to an already populated `mwalibTimeStep` array
 *
 * * `timesteps_len` - number of elements in the pointed to array
 *
 *
 * # Returns
 *
 * * 0 on success, non-zero on failure
 *
 *
 * # Safety
 * * This must be called once caller is finished with the `mwalibTimeStep` array
 * * `timestep_ptr` must point to a populated `mwalibTimeStep` array from the `mwalib_correlator_timesteps_get` function.
 * * `timestep_ptr` must not have already been freed.
 */
int32_t mwalib_timesteps_free(mwalibTimeStep *timesteps_ptr,
                              uintptr_t timesteps_len);

/**
 * This passes back a pointer to an array of all visibility polarisations
 *
 * # Arguments
 *
 * * `correlator_context_ptr` - pointer to an already populated `CorrelatorContext` object.
 *
 * * `out_visibility_pols_ptr` - A Rust-owned populated array of `mwalibVisibilityPol` structs. Free with `mwalib_visibility_pols_free`.
 *
 * * `out_visibility_pols_len` - Visibility Pols array length.
 *
 * * `error_message` - pointer to already allocated buffer for any error messages to be returned to the caller.
 *
 * * `error_message_length` - length of error_message char* buffer.
 *
 *
 * # Returns
 *
 * * 0 on success, non-zero on failure
 *
 *
 * # Safety
 * * `error_message` *must* point to an already allocated char* buffer for any error messages.
 * * `correlator_context_ptr` must point to a populated `CorrelatorContext` object from the `mwalib_correlator_context_new` function.
 * * Caller must call `mwalib_visibility_pols_free` once finished, to free the rust memory.
 */
int32_t mwalib_correlator_visibility_pols_get(CorrelatorContext *correlator_context_ptr,
                                              mwalibVisibilityPol **out_visibility_pols_ptr,
                                              uintptr_t *out_visibility_pols_len,
                                              const char *error_message,
                                              size_t error_message_length);

/**
 * Free a previously-allocated `mwalibVisibilityPol` array of structs.
 *
 * # Arguments
 *
 * * `visibility_pols_ptr` - pointer to an already populated `mwalibVisibilityPol` array
 *
 * * `visibility_pols_len` - number of elements in the pointed to array
 *
 * # Returns
 *
 * * 0 on success, non-zero on failure
 *
 *
 * # Safety
 * * This must be called once caller is finished with the `mwalibVisibilityPol` array
 * * `visibility_pols_ptr` must point to a populated `mwalibVisibilityPol` array from the `mwalib_correlator_visibility_pols_get` function.
 * * `visibility_pols_ptr` must not have already been freed.
 */
int32_t mwalib_visibility_pols_free(mwalibVisibilityPol *visibility_pols_ptr,
                                    uintptr_t visibility_pols_len);
