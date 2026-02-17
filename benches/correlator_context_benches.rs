use criterion::{criterion_group, criterion_main, Criterion, SamplingMode};
use mwalib::CorrelatorContext;

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
    let mut buffer: Vec<f32> = vec![0.0; context.num_timestep_coarse_chan_bytes];

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
    let mut buffer: Vec<f32> = vec![0.0; context.num_timestep_coarse_chan_bytes];

    group.bench_function("read_by_frequency_into_buffer", |b| {
        b.iter(|| {
            context
                .read_by_frequency_into_buffer(ts, cc, &mut buffer)
                .expect("Error calling read_by_frequency_into_buffer");
        })
    });
}

criterion_group!(
    name = correlator_context_benches;
    config = Criterion::default().sample_size(10).with_plots();
    targets = bench_gpubox_read_by_baseline, bench_gpubox_read_by_frequency,bench_gpubox_read_by_baseline_into_buffer, bench_gpubox_read_by_frequency_into_buffer
);
criterion_main!(correlator_context_benches);
