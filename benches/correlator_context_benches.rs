use criterion::{criterion_group, criterion_main, Criterion, SamplingMode, Throughput};
use mwalib::{fitsio_sys, CorrelatorContext};
use std::ffi::CString;
use std::hint::black_box;
use std::path::{Path, PathBuf};
use std::ptr;

fn setup_test_files_1101503312() -> (String, Vec<String>) {
    let metafits_filename = String::from("test_files/1101503312_1_timestep/1101503312.metafits");

    let filenames: Vec<String> = [String::from(
        "test_files/1101503312_1_timestep/1101503312_20141201210818_gpubox01_00.fits",
    )]
    .to_vec();

    (metafits_filename, filenames)
}

fn bench_gpubox_read_by_baseline(c: &mut Criterion) {
    let (metafits_filename, filenames) = setup_test_files_1101503312();

    // Enable flat sampling
    let group = &mut c.benchmark_group("my_group");
    group.sampling_mode(SamplingMode::Flat);

    // Open a context and load in a test metafits and gpubox file
    let context = CorrelatorContext::new(&metafits_filename, &filenames)
        .expect("Error creating correlator context");

    // Bench this!
    let ts: usize = 0;
    let cc: usize = 0;

    group.bench_function("read_by_baseline", |b| {
        b.iter(|| {
            context
                .read_by_baseline(ts, cc)
                .expect("Error calling read_by_baseline");
        })
    });
}

fn bench_gpubox_read_by_baseline_into_buffer(c: &mut Criterion) {
    let (metafits_filename, filenames) = setup_test_files_1101503312();

    // Enable flat sampling
    let group = &mut c.benchmark_group("my_group");
    group.sampling_mode(SamplingMode::Flat);

    // Open a context and load in a test metafits and gpubox file
    let context = CorrelatorContext::new(&metafits_filename, &filenames)
        .expect("Error creating correlator context");

    // Bench this!
    let ts: usize = 0;
    let cc: usize = 0;
    let mut buffer: Vec<f32> = vec![0.0; context.num_timestep_coarse_chan_floats];

    group.bench_function("read_by_baseline_into_buffer", |b| {
        b.iter(|| {
            context
                .read_by_baseline_into_buffer(ts, cc, &mut buffer)
                .expect("Error calling read_by_baseline_into_buffer");
        })
    });
}

fn bench_gpubox_read_by_frequency(c: &mut Criterion) {
    let (metafits_filename, filenames) = setup_test_files_1101503312();

    // Enable flat sampling
    let group = &mut c.benchmark_group("my_group");
    group.sampling_mode(SamplingMode::Flat);

    // Open a context and load in a test metafits and gpubox file
    let context = CorrelatorContext::new(&metafits_filename, &filenames)
        .expect("Error creating correlator context");

    // Bench this!
    let ts: usize = 0;
    let cc: usize = 0;

    group.bench_function("read_by_frequency", |b| {
        b.iter(|| {
            context
                .read_by_frequency(ts, cc)
                .expect("Error calling read_by_frequency");
        })
    });
}

fn bench_gpubox_read_by_frequency_into_buffer(c: &mut Criterion) {
    let (metafits_filename, filenames) = setup_test_files_1101503312();

    // Enable flat sampling
    let group = &mut c.benchmark_group("my_group");
    group.sampling_mode(SamplingMode::Flat);

    // Open a context and load in a test metafits and gpubox file
    let context = CorrelatorContext::new(&metafits_filename, &filenames)
        .expect("Error creating correlator context");

    // Bench this!
    let ts: usize = 0;
    let cc: usize = 0;
    let mut buffer: Vec<f32> = vec![0.0; context.num_timestep_coarse_chan_floats];

    group.bench_function("read_by_frequency_into_buffer", |b| {
        b.iter(|| {
            context
                .read_by_frequency_into_buffer(ts, cc, &mut buffer)
                .expect("Error calling read_by_frequency_into_buffer");
        })
    });
}

/// The MWAX test metafits and gpubox file. The gpubox file is Rice tile compressed; real MWAX gpubox files never are,
/// so the `mwax_uncompressed` group (a copy decompressed at setup) is the realistic one.
const MWAX_METAFITS: &str = "test_files/1244973688_1_timestep/1244973688.metafits";
const MWAX_GPUBOX: &str =
    "test_files/1244973688_1_timestep/1244973688_20190619100110_ch114_000.fits";

/// The timestep and coarse channel which have data in the MWAX test file
const MWAX_TIMESTEP: usize = 0;
const MWAX_COARSE_CHAN: usize = 10;

/// The legacy test metafits and gpubox file. The gpubox file is Rice tile compressed; some real legacy gpubox files
/// are, the others are not, so there are groups for both.
const LEGACY_METAFITS: &str = "test_files/1101503312_1_timestep/1101503312.metafits";
const LEGACY_GPUBOX: &str =
    "test_files/1101503312_1_timestep/1101503312_20141201210818_gpubox01_00.fits";

/// The timestep and coarse channel which have data in the legacy test file
const LEGACY_TIMESTEP: usize = 0;
const LEGACY_COARSE_CHAN: usize = 0;

/// cfitsio status for "no error"
const FITSIO_OK: i32 = 0;

/// First HDU number in cfitsio (HDU numbers start at 1)
const FITSIO_FIRST_HDU: i32 = 1;

/// Panics with a message if a cfitsio call failed.
///
/// # Arguments
///
/// * `status` - The cfitsio status after the call.
///
/// * `what` - What the call did, for the message.
///
fn check_fitsio(status: i32, what: &str) {
    assert_eq!(
        status, FITSIO_OK,
        "cfitsio failed to {what} (status {status})"
    );
}

/// Writes an uncompressed copy of a FITS file into a directory, with the same file name. Tile compressed image HDUs are
/// decompressed with `fits_img_decompress`; the other HDUs are copied as they are. The values are not changed (no
/// scaling is applied), so reads of the copy give the same data as reads of the original.
///
/// # Arguments
///
/// * `source` - Path of the FITS file to copy.
///
/// * `dest_dir` - Directory to write the copy into.
///
/// # Returns
///
/// * The path of the copy.
///
fn write_uncompressed_copy(source: &str, dest_dir: &Path) -> PathBuf {
    let dest = dest_dir.join(
        Path::new(source)
            .file_name()
            .expect("source should have a file name"),
    );
    let source_c = CString::new(source).expect("source should not contain a NUL");
    let dest_c = CString::new(dest.to_str().expect("dest should be UTF-8"))
        .expect("dest should not contain a NUL");

    let mut status: i32 = FITSIO_OK;
    let mut in_fptr: *mut fitsio_sys::fitsfile = ptr::null_mut();
    let mut out_fptr: *mut fitsio_sys::fitsfile = ptr::null_mut();
    let mut num_hdus: i32 = 0;

    // SAFETY: plain cfitsio calls on pointers cfitsio gave us; the status is checked after each call.
    unsafe {
        fitsio_sys::ffopen(
            &mut in_fptr,
            source_c.as_ptr(),
            fitsio_sys::READONLY as i32,
            &mut status,
        );
        check_fitsio(status, "open the source");
        fitsio_sys::ffinit(&mut out_fptr, dest_c.as_ptr(), &mut status);
        check_fitsio(status, "create the copy");
        fitsio_sys::ffthdu(in_fptr, &mut num_hdus, &mut status);
        check_fitsio(status, "count the HDUs");

        for hdu in FITSIO_FIRST_HDU..=num_hdus {
            fitsio_sys::ffmahd(in_fptr, hdu, ptr::null_mut(), &mut status);
            check_fitsio(status, "move to an HDU");

            if fitsio_sys::fits_is_compressed_image(in_fptr, &mut status) != 0 {
                fitsio_sys::fits_img_decompress(in_fptr, out_fptr, &mut status);
            } else {
                fitsio_sys::ffcopy(in_fptr, out_fptr, 0, &mut status);
            }
            check_fitsio(status, "copy an HDU");
        }

        fitsio_sys::ffclos(out_fptr, &mut status);
        check_fitsio(status, "close the copy");
        fitsio_sys::ffclos(in_fptr, &mut status);
        check_fitsio(status, "close the source");
    }

    dest
}

/// Benchmarks of reading a timestep and coarse channel from a gpubox file (128 tiles, so 8256 baselines and 128 fine
/// channels: about 34 MB of visibilities and 132 KB of weights).
///
/// Notes on what these measure:
/// * The file is read over and over, so after the first read it is in the operating system's file cache. These
///   benchmarks measure opening the file, cfitsio reading (and decompressing, if the file is compressed) and
///   converting the floats, and our own conversion and allocation. They do not measure reading from disk.
/// * The result of each read is dropped inside the timed code, so the cost of allocating and freeing the results is
///   included. The `_into_buffer` benchmarks reuse a buffer, so the difference between a read and its
///   `_into_buffer` version is the cost of allocation.
/// * The weights benchmark is a small read, so it is mostly the fixed cost of a read: opening the file and moving to
///   the HDU.
///
/// # Arguments
///
/// * `c` - Criterion.
///
/// * `group_name` - Name of the benchmark group.
///
/// * `context` - Context of the files to read.
///
/// * `timestep` - Timestep index to read.
///
/// * `coarse_chan` - Coarse channel index to read.
///
fn bench_reads(
    c: &mut Criterion,
    group_name: &str,
    context: &CorrelatorContext,
    timestep: usize,
    coarse_chan: usize,
) {
    // Enable flat sampling
    let mut group = c.benchmark_group(group_name);
    group.sampling_mode(SamplingMode::Flat);

    // Report the number of bytes of visibilities that are produced per second
    group.throughput(Throughput::Bytes(
        context.num_timestep_coarse_chan_bytes as u64,
    ));

    group.bench_function("read_by_baseline", |b| {
        b.iter(|| {
            black_box(
                context
                    .read_by_baseline(timestep, coarse_chan)
                    .expect("Error calling read_by_baseline"),
            );
        })
    });

    group.bench_function("read_by_frequency", |b| {
        b.iter(|| {
            black_box(
                context
                    .read_by_frequency(timestep, coarse_chan)
                    .expect("Error calling read_by_frequency"),
            );
        })
    });

    let mut buffer: Vec<f32> = vec![0.0; context.num_timestep_coarse_chan_floats];

    group.bench_function("read_by_baseline_into_buffer", |b| {
        b.iter(|| {
            context
                .read_by_baseline_into_buffer(timestep, coarse_chan, &mut buffer)
                .expect("Error calling read_by_baseline_into_buffer");
            black_box(&buffer);
        })
    });

    group.bench_function("read_by_frequency_into_buffer", |b| {
        b.iter(|| {
            context
                .read_by_frequency_into_buffer(timestep, coarse_chan, &mut buffer)
                .expect("Error calling read_by_frequency_into_buffer");
            black_box(&buffer);
        })
    });

    // Weights are much smaller than visibilities
    group.throughput(Throughput::Bytes(
        context.num_timestep_coarse_chan_weight_floats as u64 * 4,
    ));

    group.bench_function("read_weights_by_baseline", |b| {
        b.iter(|| {
            black_box(
                context
                    .read_weights_by_baseline(timestep, coarse_chan)
                    .expect("Error calling read_weights_by_baseline"),
            );
        })
    });

    let mut weights_buffer: Vec<f32> = vec![0.0; context.num_timestep_coarse_chan_weight_floats];

    group.bench_function("read_weights_by_baseline_into_buffer", |b| {
        b.iter(|| {
            context
                .read_weights_by_baseline_into_buffer(timestep, coarse_chan, &mut weights_buffer)
                .expect("Error calling read_weights_by_baseline_into_buffer");
            black_box(&weights_buffer);
        })
    });

    group.finish();
}

/// Makes a context on an uncompressed copy of a gpubox file, and checks that it reads the same data as the original.
///
/// # Arguments
///
/// * `metafits` - Path of the metafits file.
///
/// * `gpubox` - Path of the (compressed) gpubox file.
///
/// * `dest_dir` - Directory to write the uncompressed copy into.
///
/// * `timestep` - Timestep index to compare.
///
/// * `coarse_chan` - Coarse channel index to compare.
///
/// # Returns
///
/// * The context on the uncompressed copy.
///
fn uncompressed_context(
    metafits: &str,
    gpubox: &str,
    dest_dir: &Path,
    timestep: usize,
    coarse_chan: usize,
) -> CorrelatorContext {
    let copy = write_uncompressed_copy(gpubox, dest_dir);

    let original =
        CorrelatorContext::new(metafits, &[gpubox]).expect("Error creating correlator context");
    let uncompressed = CorrelatorContext::new(metafits, &[copy])
        .expect("Error creating correlator context on the uncompressed copy");

    assert_eq!(
        original.read_by_baseline(timestep, coarse_chan).unwrap(),
        uncompressed
            .read_by_baseline(timestep, coarse_chan)
            .unwrap(),
        "the uncompressed copy of {gpubox} should read the same data"
    );

    uncompressed
}

/// MWAX reads from the Rice compressed test file
fn bench_mwax_reads(c: &mut Criterion) {
    let context = CorrelatorContext::new(MWAX_METAFITS, &[MWAX_GPUBOX])
        .expect("Error creating correlator context");

    bench_reads(c, "mwax", &context, MWAX_TIMESTEP, MWAX_COARSE_CHAN);
}

/// MWAX reads from an uncompressed copy of the test file (like real MWAX gpubox files)
fn bench_mwax_uncompressed_reads(c: &mut Criterion) {
    let dir = tempfile::tempdir().expect("Error creating a temp dir");
    let context = uncompressed_context(
        MWAX_METAFITS,
        MWAX_GPUBOX,
        dir.path(),
        MWAX_TIMESTEP,
        MWAX_COARSE_CHAN,
    );

    bench_reads(
        c,
        "mwax_uncompressed",
        &context,
        MWAX_TIMESTEP,
        MWAX_COARSE_CHAN,
    );
}

/// Legacy reads from an uncompressed copy of the test file
fn bench_legacy_uncompressed_reads(c: &mut Criterion) {
    let dir = tempfile::tempdir().expect("Error creating a temp dir");
    let context = uncompressed_context(
        LEGACY_METAFITS,
        LEGACY_GPUBOX,
        dir.path(),
        LEGACY_TIMESTEP,
        LEGACY_COARSE_CHAN,
    );

    bench_reads(
        c,
        "legacy_uncompressed",
        &context,
        LEGACY_TIMESTEP,
        LEGACY_COARSE_CHAN,
    );
}

criterion_group!(
    name = correlator_context_benches;
    config = Criterion::default().sample_size(10).with_plots();
    targets = bench_gpubox_read_by_baseline, bench_gpubox_read_by_frequency, bench_gpubox_read_by_baseline_into_buffer, bench_gpubox_read_by_frequency_into_buffer, bench_mwax_reads, bench_mwax_uncompressed_reads, bench_legacy_uncompressed_reads
);
criterion_main!(correlator_context_benches);
