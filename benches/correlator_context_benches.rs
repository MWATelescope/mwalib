use criterion::{criterion_group, criterion_main, Criterion, SamplingMode, Throughput};
use mwalib::CorrelatorContext;
use std::hint::black_box;

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

/// The MWAX test metafits and gpubox file
const MWAX_METAFITS: &str = "test_files/1244973688_1_timestep/1244973688.metafits";
const MWAX_GPUBOX: &str =
    "test_files/1244973688_1_timestep/1244973688_20190619100110_ch114_000.fits";

/// The timestep and coarse channel which have data in the MWAX test file
const MWAX_TIMESTEP: usize = 0;
const MWAX_COARSE_CHAN: usize = 10;

/// Benchmarks of reading a timestep and coarse channel from an MWAX gpubox file (128 tiles, so 8256 baselines and
/// 128 fine channels: about 34 MB of visibilities and 132 KB of weights).
///
/// Notes on what these measure:
/// * The file is read over and over, so after the first read it is in the operating system's file cache. These
///   benchmarks measure opening the file, cfitsio reading and converting the floats, and our own conversion and
///   allocation. They do not measure reading from disk.
/// * The result of each read is dropped inside the timed code, so the cost of allocating and freeing the results is
///   included. The `_into_buffer` benchmarks reuse a buffer, so the difference between a read and its
///   `_into_buffer` version is the cost of allocation.
/// * The weights benchmark is a small read, so it is mostly the fixed cost of a read: opening the file and moving to
///   the HDU.
fn bench_mwax_reads(c: &mut Criterion) {
    // Open a context and load in a test metafits and gpubox file
    let context = CorrelatorContext::new(MWAX_METAFITS, &[MWAX_GPUBOX])
        .expect("Error creating correlator context");

    // Enable flat sampling
    let mut group = c.benchmark_group("mwax");
    group.sampling_mode(SamplingMode::Flat);

    // Report the number of bytes of visibilities that are produced per second
    group.throughput(Throughput::Bytes(
        context.num_timestep_coarse_chan_bytes as u64,
    ));

    group.bench_function("read_by_baseline", |b| {
        b.iter(|| {
            black_box(
                context
                    .read_by_baseline(MWAX_TIMESTEP, MWAX_COARSE_CHAN)
                    .expect("Error calling read_by_baseline"),
            );
        })
    });

    group.bench_function("read_by_frequency", |b| {
        b.iter(|| {
            black_box(
                context
                    .read_by_frequency(MWAX_TIMESTEP, MWAX_COARSE_CHAN)
                    .expect("Error calling read_by_frequency"),
            );
        })
    });

    let mut buffer: Vec<f32> = vec![0.0; context.num_timestep_coarse_chan_floats];

    group.bench_function("read_by_baseline_into_buffer", |b| {
        b.iter(|| {
            context
                .read_by_baseline_into_buffer(MWAX_TIMESTEP, MWAX_COARSE_CHAN, &mut buffer)
                .expect("Error calling read_by_baseline_into_buffer");
            black_box(&buffer);
        })
    });

    group.bench_function("read_by_frequency_into_buffer", |b| {
        b.iter(|| {
            context
                .read_by_frequency_into_buffer(MWAX_TIMESTEP, MWAX_COARSE_CHAN, &mut buffer)
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
                    .read_weights_by_baseline(MWAX_TIMESTEP, MWAX_COARSE_CHAN)
                    .expect("Error calling read_weights_by_baseline"),
            );
        })
    });

    let mut weights_buffer: Vec<f32> = vec![0.0; context.num_timestep_coarse_chan_weight_floats];

    group.bench_function("read_weights_by_baseline_into_buffer", |b| {
        b.iter(|| {
            context
                .read_weights_by_baseline_into_buffer(
                    MWAX_TIMESTEP,
                    MWAX_COARSE_CHAN,
                    &mut weights_buffer,
                )
                .expect("Error calling read_weights_by_baseline_into_buffer");
            black_box(&weights_buffer);
        })
    });

    group.finish();
}

criterion_group!(
    name = correlator_context_benches;
    config = Criterion::default().sample_size(10).with_plots();
    targets = bench_gpubox_read_by_baseline, bench_gpubox_read_by_frequency, bench_gpubox_read_by_baseline_into_buffer, bench_gpubox_read_by_frequency_into_buffer, bench_mwax_reads
);
criterion_main!(correlator_context_benches);
