use super::*;

#[test]
//
// Simple test of the error message helper
//
fn test_set_error_message() {
    let buffer = CString::new("HELLO WORLD").unwrap();
    let buffer_ptr = buffer.as_ptr() as *mut u8;

    set_error_message("hello world", buffer_ptr, 12);

    assert_eq!(buffer, CString::new("hello world").unwrap());
}

//
// Metafits context Tests
//
#[test]
fn test_mwalib_metafits_context_new_valid() {
    // This tests for a valid metafitscontext
    let error_message =
        CString::new("                                                            ").unwrap();
    let error_message_ptr = error_message.as_ptr() as *const c_char;
    let error_len: size_t = 60;

    let metafits_file =
        CString::new("test_files/1101503312_1_timestep/1101503312.metafits").unwrap();
    let metafits_file_ptr = metafits_file.as_ptr();

    unsafe {
        // Create a MetafitsContext
        let mut metafits_context_ptr: *mut MetafitsContext = std::ptr::null_mut();
        let retval = mwalib_metafits_context_new(
            metafits_file_ptr,
            //gpubox_files_ptr,
            //1,
            &mut metafits_context_ptr,
            error_message_ptr,
            error_len,
        );

        // Check return value of mwalib_metafits_context_new
        assert_eq!(retval, 0, "mwalib_metafits_context_new failure");

        // Check we got valid MetafitsContext pointer
        let context_ptr = metafits_context_ptr.as_mut();
        assert!(context_ptr.is_some());
    }
}

//
// CorrelatorContext Tests
//
#[test]
fn test_mwalib_correlator_context_new_valid() {
    // This tests for a valid correlator context
    let error_message =
        CString::new("                                                            ").unwrap();
    let error_message_ptr = error_message.as_ptr() as *const c_char;
    let error_len: size_t = 60;

    let metafits_file =
        CString::new("test_files/1101503312_1_timestep/1101503312.metafits").unwrap();
    let metafits_file_ptr = metafits_file.as_ptr();

    let gpubox_file =
        CString::new("test_files/1101503312_1_timestep/1101503312_20141201210818_gpubox01_00.fits")
            .unwrap();
    let mut gpubox_files: Vec<*const c_char> = Vec::new();
    gpubox_files.push(gpubox_file.as_ptr());
    let gpubox_files_ptr = gpubox_files.as_ptr() as *mut *const c_char;

    unsafe {
        // Create a CorrelatorContext
        let mut correlator_context_ptr: *mut CorrelatorContext = std::ptr::null_mut();
        let retval = mwalib_correlator_context_new(
            metafits_file_ptr,
            gpubox_files_ptr,
            1,
            &mut correlator_context_ptr,
            error_message_ptr,
            error_len,
        );

        // Check return value of mwalib_correlator_context_new
        assert_eq!(retval, 0, "mwalib_correlator_context_new failure");

        // Check we got valid MetafitsContext pointer
        let context_ptr = correlator_context_ptr.as_mut();
        assert!(context_ptr.is_some());
    }
}

//
// Metafits Metadata Tests
//
#[test]
fn test_mwalib_metafits_metadata_get_from_metafits_context_valid() {
    // This tests for a valid metafits context and metadata returned

    let error_message =
        CString::new("                                                            ").unwrap();
    let error_message_ptr = error_message.as_ptr() as *const c_char;
    let error_len: size_t = 60;

    let metafits_file =
        CString::new("test_files/1101503312_1_timestep/1101503312.metafits").unwrap();
    let metafits_file_ptr = metafits_file.as_ptr();

    unsafe {
        // Create a MetafitsContext
        let mut metafits_context_ptr: *mut MetafitsContext = std::ptr::null_mut();
        let mut retval = mwalib_metafits_context_new(
            metafits_file_ptr,
            //gpubox_files_ptr,
            //1,
            &mut metafits_context_ptr,
            error_message_ptr,
            error_len,
        );

        // Check return value of mwalib_metafits_context_new
        assert_eq!(retval, 0, "mwalib_metafits_context_new failure");

        // Check we got valid MetafitsContext pointer
        let context_ptr = metafits_context_ptr.as_mut();
        assert!(context_ptr.is_some());

        // Populate a mwalibMetafitsMetadata struct
        let mut metafits_metadata_ptr: &mut *mut mwalibMetafitsMetadata = &mut std::ptr::null_mut();
        retval = mwalib_metafits_metadata_get(
            metafits_context_ptr,
            std::ptr::null_mut(),
            &mut metafits_metadata_ptr,
            error_message_ptr,
            error_len,
        );

        // Check return value
        assert_eq!(retval, 0, "mwalib_metafits_metadata_get failure");

        // Get the mwalibMetadata struct from the pointer
        let metafits_metadata = Box::from_raw(*metafits_metadata_ptr);

        // We should get a valid timestep and no error message
        assert_eq!(metafits_metadata.obsid, 1_101_503_312);
    }
}

#[test]
fn test_mwalib_metafits_metadata_get_from_correlator_context_valid() {
    // This tests for a valid metafits metadata returned given a correlator context
    let error_message =
        CString::new("                                                            ").unwrap();
    let error_message_ptr = error_message.as_ptr() as *const c_char;
    let error_len: size_t = 60;

    let metafits_file =
        CString::new("test_files/1101503312_1_timestep/1101503312.metafits").unwrap();
    let metafits_file_ptr = metafits_file.as_ptr();

    let gpubox_file =
        CString::new("test_files/1101503312_1_timestep/1101503312_20141201210818_gpubox01_00.fits")
            .unwrap();
    let mut gpubox_files: Vec<*const c_char> = Vec::new();
    gpubox_files.push(gpubox_file.as_ptr());
    let gpubox_files_ptr = gpubox_files.as_ptr() as *mut *const c_char;

    unsafe {
        // Create a CorrelatorContext
        let mut correlator_context_ptr: *mut CorrelatorContext = std::ptr::null_mut();
        let mut retval = mwalib_correlator_context_new(
            metafits_file_ptr,
            gpubox_files_ptr,
            1,
            &mut correlator_context_ptr,
            error_message_ptr,
            error_len,
        );

        // Check return value of mwalib_correlator_context_new
        assert_eq!(retval, 0, "mwalib_correlator_context_new failure");

        // Check we got valid MetafitsContext pointer
        let context_ptr = correlator_context_ptr.as_mut();
        assert!(context_ptr.is_some());

        // Populate a mwalibMetafitsMetadata struct
        let mut metafits_metadata_ptr: &mut *mut mwalibMetafitsMetadata = &mut std::ptr::null_mut();
        retval = mwalib_metafits_metadata_get(
            std::ptr::null_mut(),
            correlator_context_ptr,
            &mut metafits_metadata_ptr,
            error_message_ptr,
            error_len,
        );

        // Check return value
        assert_eq!(retval, 0, "mwalib_metafits_metadata_get failure");

        // Get the mwalibMetadata struct from the pointer
        let metafits_metadata = Box::from_raw(*metafits_metadata_ptr);

        // We should get a valid timestep and no error message
        assert_eq!(metafits_metadata.obsid, 1_101_503_312);
    }
}

#[test]
fn test_mwalib_correlator_metadata_get_valid() {
    // This tests for a valid correlator metadata
    let error_message =
        CString::new("                                                            ").unwrap();
    let error_message_ptr = error_message.as_ptr() as *mut c_char;
    let error_len: size_t = 60;

    let metafits_file =
        CString::new("test_files/1101503312_1_timestep/1101503312.metafits").unwrap();
    let metafits_file_ptr = metafits_file.as_ptr();

    let gpubox_file =
        CString::new("test_files/1101503312_1_timestep/1101503312_20141201210818_gpubox01_00.fits")
            .unwrap();
    let mut gpubox_files: Vec<*const c_char> = Vec::new();
    gpubox_files.push(gpubox_file.as_ptr());
    let gpubox_files_ptr = gpubox_files.as_ptr() as *mut *const c_char;

    unsafe {
        // Create a CorrelatorContext
        let mut correlator_context_ptr: *mut CorrelatorContext = std::ptr::null_mut();
        let mut retval = mwalib_correlator_context_new(
            metafits_file_ptr,
            gpubox_files_ptr,
            1,
            &mut correlator_context_ptr,
            error_message_ptr,
            60,
        );

        // Check return value of mwalib_correlator_context_new
        assert_eq!(retval, 0, "mwalib_correlator_context_new failure");

        // Check we got valid MetafitsContext pointer
        let context_ptr = correlator_context_ptr.as_mut();
        assert!(context_ptr.is_some());

        // Populate a mwalibCorrelatorMetadata struct
        let mut correlator_metadata_ptr: &mut *mut mwalibCorrelatorMetadata =
            &mut std::ptr::null_mut();
        retval = mwalib_correlator_metadata_get(
            correlator_context_ptr,
            &mut correlator_metadata_ptr,
            error_message_ptr,
            error_len,
        );

        // Check return value
        assert_eq!(retval, 0, "mwalib_correlator_metadata_get failure");

        // Get the mwalibMetadata struct from the pointer
        let correlator_metadata = Box::from_raw(*correlator_metadata_ptr);

        // We should get a valid timestep and no error message
        assert_eq!(correlator_metadata.integration_time_milliseconds, 2000);
    }
}
/*
#[test]
fn test_mwalibmetadata_get_null_context() {
    // This tests for a null context
    let error_message =
        CString::new("                                                            ").unwrap();
    let error_message_ptr = error_message.as_ptr() as *mut u8;
    let error_len: size_t = 60;

    unsafe {
        let context_ptr = std::ptr::null_mut();
        let md_ptr = mwalib_metafits_metadata_get(context_ptr, error_message_ptr, error_len);

        // We should get a null pointer and an error message
        assert!(md_ptr.is_null());
        let expected_error: &str = &"mwalib_metafits_metadata_get() ERROR:";
        assert_eq!(
            error_message.into_string().unwrap()[0..expected_error.len()],
            *expected_error
        );
    }
}

// RF Input
#[test]
fn test_mwalib_rfinput_get_valid() {
    // This tests for a valid context with a valid timestmwalibRFInputep
    let rf_index = 2; // valid  should be Tile012(X)

    let error_message =
        CString::new("                                                            ").unwrap();
    let error_message_ptr = error_message.as_ptr() as *mut u8;
    let error_len: size_t = 60;

    let metafits_file =
        CString::new("test_files/1101503312_1_timestep/1101503312.metafits").unwrap();
    let metafits_file_ptr = metafits_file.as_ptr();

    /*let gpubox_file = CString::new(
        "test_files/1101503312_1_timestep/1101503312_20141201210818_gpubox01_00.fits",
    )
    .unwrap();
    let mut gpubox_files: Vec<*const c_char> = Vec::new();
    gpubox_files.push(gpubox_file.as_ptr());
    let gpubox_files_ptr = gpubox_files.as_ptr() as *mut *const c_char;*/

    unsafe {
        let context = mwalib_metafits_context_new(
            metafits_file_ptr,
            //gpubox_files_ptr,
            //1,
            error_message_ptr,
            60,
        );

        // Check we got a context object
        let context_ptr = context.as_mut();
        assert!(context_ptr.is_some());

        let rf = Box::from_raw(mwalib_rfinput_get(
            context,
            rf_index,
            error_message_ptr,
            error_len,
        ));

        // We should get a valid timestep and no error message
        assert_eq!(rf.antenna, 1);

        assert_eq!(
            CString::from_raw(rf.tile_name),
            CString::new("Tile012").unwrap()
        );

        assert_eq!(CString::from_raw(rf.pol), CString::new("X").unwrap());

        let expected_error: &str = &"mwalib_rfinput_get() ERROR:";
        assert_ne!(
            error_message.into_string().unwrap()[0..expected_error.len()],
            *expected_error
        );
    }
}

#[test]
fn test_mwalib_rfinput_get_invalid() {
    // This tests for a valid context with an invalid mwalibRFInput (out of bounds)
    let rf_index = 300; // invalid

    let error_message =
        CString::new("                                                            ").unwrap();
    let error_message_ptr = error_message.as_ptr() as *mut u8;
    let error_len: size_t = 60;

    let metafits_file =
        CString::new("test_files/1101503312_1_timestep/1101503312.metafits").unwrap();
    let metafits_file_ptr = metafits_file.as_ptr();

    /*let gpubox_file = CString::new(
        "test_files/1101503312_1_timestep/1101503312_20141201210818_gpubox01_00.fits",
    )
    .unwrap();
    let mut gpubox_files: Vec<*const c_char> = Vec::new();
    gpubox_files.push(gpubox_file.as_ptr());
    let gpubox_files_ptr = gpubox_files.as_ptr() as *mut *const c_char;*/

    unsafe {
        let context = mwalib_metafits_context_new(
            metafits_file_ptr,
            //gpubox_files_ptr,
            //1,
            error_message_ptr,
            60,
        );

        // Check we got a context object
        let context_ptr = context.as_mut();
        assert!(context_ptr.is_some());

        let rf_ptr = mwalib_rfinput_get(context, rf_index, error_message_ptr, error_len);

        // We should get a null pointer and an error message
        assert!(rf_ptr.is_null());
        let expected_error: &str = &"mwalib_rfinput_get() ERROR:";
        assert_eq!(
            error_message.into_string().unwrap()[0..expected_error.len()],
            *expected_error
        );
    }
}

#[test]
fn test_mwalib_rfinput_get_null_context() {
    // This tests for a null context with an valid mwalibRFInput
    let rf_index = 100; // valid

    let error_message =
        CString::new("                                                            ").unwrap();
    let error_message_ptr = error_message.as_ptr() as *mut u8;
    let error_len: size_t = 60;

    unsafe {
        let context_ptr = std::ptr::null_mut();
        let rf_ptr = mwalib_rfinput_get(context_ptr, rf_index, error_message_ptr, error_len);

        // We should get a null pointer and an error message
        assert!(rf_ptr.is_null());
        let expected_error: &str = &"mwalib_rfinput_get() ERROR:";
        assert_eq!(
            error_message.into_string().unwrap()[0..expected_error.len()],
            *expected_error
        );
    }
}

// Coarse Channel
#[test]
fn test_mwalib_coarsechannel_get_valid() {
    // This tests for a valid context with a valid mwalibCoarseChannel
    let channel_index = 0; // valid

    let error_message =
        CString::new("                                                            ").unwrap();
    let error_message_ptr = error_message.as_ptr() as *mut u8;
    let error_len: size_t = 60;

    let metafits_file =
        CString::new("test_files/1101503312_1_timestep/1101503312.metafits").unwrap();
    let metafits_file_ptr = metafits_file.as_ptr();

    let gpubox_file = CString::new(
        "test_files/1101503312_1_timestep/1101503312_20141201210818_gpubox01_00.fits",
    )
    .unwrap();
    let mut gpubox_files: Vec<*const c_char> = Vec::new();
    gpubox_files.push(gpubox_file.as_ptr());
    let gpubox_files_ptr = gpubox_files.as_ptr() as *mut *const c_char;

    unsafe {
        let context = mwalib_correlator_context_new(
            metafits_file_ptr,
            gpubox_files_ptr,
            1,
            error_message_ptr,
            60,
        );

        // Check we got a context object
        let context_ptr = context.as_mut();
        assert!(context_ptr.is_some());

        let ch = Box::from_raw(mwalib_correlator_coarse_channel_get(
            context,
            channel_index,
            error_message_ptr,
            error_len,
        ));

        // We should get a valid timestep and no error message
        assert_eq!(ch.receiver_channel_number, 109);

        let expected_error: &str = &"mwalib_correlator_coarse_channel_get() ERROR:";
        assert_ne!(
            error_message.into_string().unwrap()[0..expected_error.len()],
            *expected_error
        );
    }
}

#[test]
fn test_mwalib_coarsechannel_get_invalid() {
    // This tests for a valid context with an invalid mwalibCoarseChannel (out of bounds)
    let chan_index = 100; // invalid

    let error_message =
        CString::new("                                                            ").unwrap();
    let error_message_ptr = error_message.as_ptr() as *mut u8;
    let error_len: size_t = 60;

    let metafits_file =
        CString::new("test_files/1101503312_1_timestep/1101503312.metafits").unwrap();
    let metafits_file_ptr = metafits_file.as_ptr();

    let gpubox_file = CString::new(
        "test_files/1101503312_1_timestep/1101503312_20141201210818_gpubox01_00.fits",
    )
    .unwrap();
    let mut gpubox_files: Vec<*const c_char> = Vec::new();
    gpubox_files.push(gpubox_file.as_ptr());
    let gpubox_files_ptr = gpubox_files.as_ptr() as *mut *const c_char;

    unsafe {
        let context = mwalib_correlator_context_new(
            metafits_file_ptr,
            gpubox_files_ptr,
            1,
            error_message_ptr,
            60,
        );

        // Check we got a context object
        let context_ptr = context.as_mut();
        assert!(context_ptr.is_some());

        let ch_ptr = mwalib_correlator_coarse_channel_get(
            context,
            chan_index,
            error_message_ptr,
            error_len,
        );

        // We should get a null pointer and an error message
        assert!(ch_ptr.is_null());
        let expected_error: &str = &"mwalib_correlator_coarse_channel_get() ERROR:";
        assert_eq!(
            error_message.into_string().unwrap()[0..expected_error.len()],
            *expected_error
        );
    }
}

#[test]
fn test_mwalibcoarsechannel_get_null_context() {
    // This tests for a null context with a valid mwalibCoarseChannel
    let timestep_index = 0; // valid

    let error_message =
        CString::new("                                                            ").unwrap();
    let error_message_ptr = error_message.as_ptr() as *mut u8;
    let error_len: size_t = 60;

    unsafe {
        let context_ptr = std::ptr::null_mut();
        let ch_ptr = mwalib_correlator_coarse_channel_get(
            context_ptr,
            timestep_index,
            error_message_ptr,
            error_len,
        );

        // We should get a null pointer and an error message
        assert!(ch_ptr.is_null());
        let expected_error: &str = &"mwalib_correlator_coarse_channel_get() ERROR:";
        assert_eq!(
            error_message.into_string().unwrap()[0..expected_error.len()],
            *expected_error
        );
    }
}

// Antenna
#[test]
fn test_mwalibantenna_get_valid() {
    // This tests for a valid context with a valid mwalibAntenna
    let ant_index = 2; // valid- should be Tile013

    let error_message =
        CString::new("                                                            ").unwrap();
    let error_message_ptr = error_message.as_ptr() as *mut u8;
    let error_len: size_t = 60;

    let metafits_file =
        CString::new("test_files/1101503312_1_timestep/1101503312.metafits").unwrap();
    let metafits_file_ptr = metafits_file.as_ptr();

    /*let gpubox_file = CString::new(
        "test_files/1101503312_1_timestep/1101503312_20141201210818_gpubox01_00.fits",
    )
    .unwrap();
    let mut gpubox_files: Vec<*const c_char> = Vec::new();
    gpubox_files.push(gpubox_file.as_ptr());
    let gpubox_files_ptr = gpubox_files.as_ptr() as *mut *const c_char;*/

    unsafe {
        let context = mwalib_metafits_context_new(
            metafits_file_ptr,
            //gpubox_files_ptr,
            //1,
            error_message_ptr,
            60,
        );

        // Check we got a context object
        let context_ptr = context.as_mut();
        assert!(context_ptr.is_some());

        let ant = Box::from_raw(mwalib_antenna_get(
            context,
            ant_index,
            error_message_ptr,
            error_len,
        ));

        // We should get a valid timestep and no error message
        assert_eq!(ant.tile_id, 13);

        let expected_error: &str = &"mwalib_antenna_get() ERROR:";
        assert_ne!(
            error_message.into_string().unwrap()[0..expected_error.len()],
            *expected_error
        );
    }
}

#[test]
fn test_mwalibantenna_get_invalid() {
    // This tests for a valid context with an invalid mwalibAntenna (out of bounds)
    let ant_index = 300; // invalid

    let error_message =
        CString::new("                                                            ").unwrap();
    let error_message_ptr = error_message.as_ptr() as *mut u8;
    let error_len: size_t = 60;

    let metafits_file =
        CString::new("test_files/1101503312_1_timestep/1101503312.metafits").unwrap();
    let metafits_file_ptr = metafits_file.as_ptr();

    /*let gpubox_file = CString::new(
        "test_files/1101503312_1_timestep/1101503312_20141201210818_gpubox01_00.fits",
    )
    .unwrap();
    let mut gpubox_files: Vec<*const c_char> = Vec::new();
    gpubox_files.push(gpubox_file.as_ptr());
    let gpubox_files_ptr = gpubox_files.as_ptr() as *mut *const c_char;*/

    unsafe {
        let context = mwalib_metafits_context_new(
            metafits_file_ptr,
            //gpubox_files_ptr,
            //1,
            error_message_ptr,
            60,
        );

        // Check we got a context object
        let context_ptr = context.as_mut();
        assert!(context_ptr.is_some());

        let ant_ptr = mwalib_antenna_get(context, ant_index, error_message_ptr, error_len);

        // We should get a null pointer and an error message
        assert!(ant_ptr.is_null());
        let expected_error: &str = &"mwalib_antenna_get() ERROR:";
        assert_eq!(
            error_message.into_string().unwrap()[0..expected_error.len()],
            *expected_error
        );
    }
}

#[test]
fn test_mwalibantenna_get_null_context() {
    // This tests for a null context with an valid mwalibAntenna
    let ant_index = 2; // valid

    let error_message =
        CString::new("                                                            ").unwrap();
    let error_message_ptr = error_message.as_ptr() as *mut u8;
    let error_len: size_t = 60;

    unsafe {
        let context_ptr = std::ptr::null_mut();
        let ant_ptr = mwalib_antenna_get(context_ptr, ant_index, error_message_ptr, error_len);

        // We should get a null pointer and an error message
        assert!(ant_ptr.is_null());
        let expected_error: &str = &"mwalib_antenna_get() ERROR:";
        assert_eq!(
            error_message.into_string().unwrap()[0..expected_error.len()],
            *expected_error
        );
    }
}

// baseline
#[test]
fn test_mwalib_baseline_get_valid() {
    // This tests for a valid context with a valid baseline
    let baseline_index = 2; // valid

    let error_message =
        CString::new("                                                            ").unwrap();
    let error_message_ptr = error_message.as_ptr() as *mut u8;
    let error_len: size_t = 60;

    let metafits_file =
        CString::new("test_files/1101503312_1_timestep/1101503312.metafits").unwrap();
    let metafits_file_ptr = metafits_file.as_ptr();

    let gpubox_file = CString::new(
        "test_files/1101503312_1_timestep/1101503312_20141201210818_gpubox01_00.fits",
    )
    .unwrap();
    let mut gpubox_files: Vec<*const c_char> = Vec::new();
    gpubox_files.push(gpubox_file.as_ptr());
    let gpubox_files_ptr = gpubox_files.as_ptr() as *mut *const c_char;

    unsafe {
        let context = mwalib_correlator_context_new(
            metafits_file_ptr,
            gpubox_files_ptr,
            1,
            error_message_ptr,
            60,
        );

        // Check we got a context object
        let context_ptr = context.as_mut();
        assert!(context_ptr.is_some());

        let bl = Box::from_raw(mwalib_baseline_get(
            context,
            baseline_index,
            error_message_ptr,
            error_len,
        ));

        // We should get a valid baseline and no error message
        assert_eq!(bl.antenna1_index, 0);
        assert_eq!(bl.antenna2_index, 2);

        let expected_error: &str = &"mwalib_baseline_get() ERROR:";
        assert_ne!(
            error_message.into_string().unwrap()[0..expected_error.len()],
            *expected_error
        );
    }
}

#[test]
fn test_mwalib_baseline_get_invalid() {
    // This tests for a valid context with an invalid baseline (out of bounds)
    let baseline_index = 100_000; // invalid

    let error_message =
        CString::new("                                                            ").unwrap();
    let error_message_ptr = error_message.as_ptr() as *mut u8;
    let error_len: size_t = 60;

    let metafits_file =
        CString::new("test_files/1101503312_1_timestep/1101503312.metafits").unwrap();
    let metafits_file_ptr = metafits_file.as_ptr();

    let gpubox_file = CString::new(
        "test_files/1101503312_1_timestep/1101503312_20141201210818_gpubox01_00.fits",
    )
    .unwrap();
    let mut gpubox_files: Vec<*const c_char> = Vec::new();
    gpubox_files.push(gpubox_file.as_ptr());
    let gpubox_files_ptr = gpubox_files.as_ptr() as *mut *const c_char;

    unsafe {
        let context = mwalib_correlator_context_new(
            metafits_file_ptr,
            gpubox_files_ptr,
            1,
            error_message_ptr,
            60,
        );

        // Check we got a context object
        let context_ptr = context.as_mut();
        assert!(context_ptr.is_some());

        let bl_ptr = mwalib_baseline_get(context, baseline_index, error_message_ptr, error_len);

        // We should get a null pointer and an error message
        assert!(bl_ptr.is_null());
        let expected_error: &str = &"mwalib_baseline_get() ERROR:";
        assert_eq!(
            error_message.into_string().unwrap()[0..expected_error.len()],
            *expected_error
        );
    }
}

#[test]
fn test_mwalibbaseline_get_null_context() {
    // This tests for a null context with an valid baseline
    let baseline_index = 1; // valid

    let error_message =
        CString::new("                                                            ").unwrap();
    let error_message_ptr = error_message.as_ptr() as *mut u8;
    let error_len: size_t = 60;

    unsafe {
        let context_ptr = std::ptr::null_mut();
        let bl_ptr =
            mwalib_baseline_get(context_ptr, baseline_index, error_message_ptr, error_len);

        // We should get a null pointer and an error message
        assert!(bl_ptr.is_null());
        let expected_error: &str = &"mwalib_baseline_get() ERROR:";
        assert_eq!(
            error_message.into_string().unwrap()[0..expected_error.len()],
            *expected_error
        );
    }
}

// timestep
#[test]
fn test_mwalibtimestep_get_valid() {
    // This tests for a valid context with a valid timestep
    let timestep_index = 0; // valid

    let error_message =
        CString::new("                                                            ").unwrap();
    let error_message_ptr = error_message.as_ptr() as *mut u8;
    let error_len: size_t = 60;

    let metafits_file =
        CString::new("test_files/1101503312_1_timestep/1101503312.metafits").unwrap();
    let metafits_file_ptr = metafits_file.as_ptr();

    let gpubox_file = CString::new(
        "test_files/1101503312_1_timestep/1101503312_20141201210818_gpubox01_00.fits",
    )
    .unwrap();
    let mut gpubox_files: Vec<*const c_char> = Vec::new();
    gpubox_files.push(gpubox_file.as_ptr());
    let gpubox_files_ptr = gpubox_files.as_ptr() as *mut *const c_char;

    unsafe {
        let context = mwalib_correlator_context_new(
            metafits_file_ptr,
            gpubox_files_ptr,
            1,
            error_message_ptr,
            60,
        );

        // Check we got a context object
        let context_ptr = context.as_mut();
        assert!(context_ptr.is_some());

        let ts = Box::from_raw(mwalib_correlator_timestep_get(
            context,
            timestep_index,
            error_message_ptr,
            error_len,
        ));

        // We should get a valid timestep and no error message
        assert_eq!(ts.unix_time_ms, 1_417_468_096_000);

        let expected_error: &str = &"mwalib_correlator_timestep_get() ERROR:";
        assert_ne!(
            error_message.into_string().unwrap()[0..expected_error.len()],
            *expected_error
        );
    }
}

#[test]
fn test_mwalibtimestep_get_invalid() {
    // This tests for a valid context with an invalid timestep (out of bounds)
    let timestep_index = 100; // invalid

    let error_message =
        CString::new("                                                            ").unwrap();
    let error_message_ptr = error_message.as_ptr() as *mut u8;
    let error_len: size_t = 60;

    let metafits_file =
        CString::new("test_files/1101503312_1_timestep/1101503312.metafits").unwrap();
    let metafits_file_ptr = metafits_file.as_ptr();

    let gpubox_file = CString::new(
        "test_files/1101503312_1_timestep/1101503312_20141201210818_gpubox01_00.fits",
    )
    .unwrap();
    let mut gpubox_files: Vec<*const c_char> = Vec::new();
    gpubox_files.push(gpubox_file.as_ptr());
    let gpubox_files_ptr = gpubox_files.as_ptr() as *mut *const c_char;

    unsafe {
        let context = mwalib_correlator_context_new(
            metafits_file_ptr,
            gpubox_files_ptr,
            1,
            error_message_ptr,
            60,
        );

        // Check we got a context object
        let context_ptr = context.as_mut();
        assert!(context_ptr.is_some());

        let ts_ptr = mwalib_correlator_timestep_get(
            context,
            timestep_index,
            error_message_ptr,
            error_len,
        );

        // We should get a null pointer and an error message
        assert!(ts_ptr.is_null());
        let expected_error: &str = &"mwalib_correlator_timestep_get() ERROR:";
        assert_eq!(
            error_message.into_string().unwrap()[0..expected_error.len()],
            *expected_error
        );
    }
}

#[test]
fn test_mwalibtimestep_get_null_context() {
    // This tests for a null context with an valid timestep
    let timestep_index = 0; // valid

    let error_message =
        CString::new("                                                            ").unwrap();
    let error_message_ptr = error_message.as_ptr() as *mut u8;
    let error_len: size_t = 60;

    unsafe {
        let context_ptr = std::ptr::null_mut();
        let ts_ptr = mwalib_correlator_timestep_get(
            context_ptr,
            timestep_index,
            error_message_ptr,
            error_len,
        );

        // We should get a null pointer and an error message
        assert!(ts_ptr.is_null());
        let expected_error: &str = &"mwalib_correlator_timestep_get() ERROR:";
        assert_eq!(
            error_message.into_string().unwrap()[0..expected_error.len()],
            *expected_error
        );
    }
}

// visibilitypol
#[test]
fn test_mwalib_correlator_visibility_pol_get_valid() {
    // This tests for a valid context with a valid visibilitypol
    let vispol_index = 0; // valid

    let error_message =
        CString::new("                                                            ").unwrap();
    let error_message_ptr = error_message.as_ptr() as *mut u8;
    let error_len: size_t = 60;

    let metafits_file =
        CString::new("test_files/1101503312_1_timestep/1101503312.metafits").unwrap();
    let metafits_file_ptr = metafits_file.as_ptr();

    let gpubox_file = CString::new(
        "test_files/1101503312_1_timestep/1101503312_20141201210818_gpubox01_00.fits",
    )
    .unwrap();
    let mut gpubox_files: Vec<*const c_char> = Vec::new();
    gpubox_files.push(gpubox_file.as_ptr());
    let gpubox_files_ptr = gpubox_files.as_ptr() as *mut *const c_char;

    unsafe {
        let context = mwalib_correlator_context_new(
            metafits_file_ptr,
            gpubox_files_ptr,
            1,
            error_message_ptr,
            60,
        );

        // Check we got a context object
        let context_ptr = context.as_mut();
        assert!(context_ptr.is_some());

        let vp = Box::from_raw(mwalib_correlator_visibility_pol_get(
            context,
            vispol_index,
            error_message_ptr,
            error_len,
        ));

        // We should get a valid timestep and no error message
        assert_eq!(
            CString::from_raw(vp.polarisation).into_string().unwrap(),
            String::from("XX")
        );

        let expected_error: &str = &"mwalib_correlator_visibility_pol_get() ERROR:";
        assert_ne!(
            error_message.into_string().unwrap()[0..expected_error.len()],
            *expected_error
        );
    }
}

#[test]
fn test_mwalib_correlator_visibility_pol_get_invalid() {
    // This tests for a valid context with an invalid visibility pol (out of bounds)
    let vispol_index = 100; // invalid

    let error_message =
        CString::new("                                                            ").unwrap();
    let error_message_ptr = error_message.as_ptr() as *mut u8;
    let error_len: size_t = 60;

    let metafits_file =
        CString::new("test_files/1101503312_1_timestep/1101503312.metafits").unwrap();
    let metafits_file_ptr = metafits_file.as_ptr();

    let gpubox_file = CString::new(
        "test_files/1101503312_1_timestep/1101503312_20141201210818_gpubox01_00.fits",
    )
    .unwrap();
    let mut gpubox_files: Vec<*const c_char> = Vec::new();
    gpubox_files.push(gpubox_file.as_ptr());
    let gpubox_files_ptr = gpubox_files.as_ptr() as *mut *const c_char;

    unsafe {
        let context = mwalib_correlator_context_new(
            metafits_file_ptr,
            gpubox_files_ptr,
            1,
            error_message_ptr,
            60,
        );

        // Check we got a context object
        let context_ptr = context.as_mut();
        assert!(context_ptr.is_some());

        let ts_ptr = mwalib_correlator_visibility_pol_get(
            context,
            vispol_index,
            error_message_ptr,
            error_len,
        );

        // We should get a null pointer and an error message
        assert!(ts_ptr.is_null());
        let expected_error: &str = &"mwalib_correlator_visibility_pol_get() ERROR:";
        assert_eq!(
            error_message.into_string().unwrap()[0..expected_error.len()],
            *expected_error
        );
    }
}

#[test]
fn test_mwalibvisibilitypol_get_null_context() {
    // This tests for a null context with an valid visibility pol
    let vispol_index = 0; // valid

    let error_message =
        CString::new("                                                            ").unwrap();
    let error_message_ptr = error_message.as_ptr() as *mut u8;
    let error_len: size_t = 60;

    unsafe {
        let context_ptr = std::ptr::null_mut();
        let ts_ptr = mwalib_correlator_visibility_pol_get(
            context_ptr,
            vispol_index,
            error_message_ptr,
            error_len,
        );

        // We should get a null pointer and an error message
        assert!(ts_ptr.is_null());
        let expected_error: &str = &"mwalib_correlator_visibility_pol_get() ERROR:";
        assert_eq!(
            error_message.into_string().unwrap()[0..expected_error.len()],
            *expected_error
        );
    }
}*/
