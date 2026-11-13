//! Benchmarks of the visibility converters on their own, on in-memory buffers.
//!
//! These separate the conversion time from the cfitsio read that `correlator_context_benches` also measures. They need
//! the internal `bench-internals` feature:
//!
//! `cargo bench --features bench-internals --bench convert_benches`
//!
//! Notes on what these measure:
//! * The input and output buffers are allocated once, outside the timed code, so no allocation is measured, except in
//!   the `reference/alloc_and_copy` benchmark.
//! * `reference/copy` is a plain copy of the same number of floats. It is the floor for any converter.
//! * `reference/alloc_and_copy` allocates a new buffer, copies into it and frees it. The difference from
//!   `reference/copy` is the cost of a new buffer of this size (allocation and first touch of its pages).
use criterion::{criterion_group, criterion_main, Criterion, SamplingMode, Throughput};
use mwalib::bench_internals::{
    convert_legacy_hdu_to_mwax_baseline_order, convert_legacy_hdu_to_mwax_frequency_order,
    convert_mwax_hdu_to_frequency_order, LegacyConversionTable,
};
use mwalib::{MWAVersion, MetafitsContext};
use std::hint::black_box;

/// The legacy test metafits (128 tiles)
const LEGACY_METAFITS: &str = "test_files/1101503312_1_timestep/1101503312.metafits";

/// The MWAX test metafits (128 tiles)
const MWAX_METAFITS: &str = "test_files/1244973688_1_timestep/1244973688.metafits";

/// Number of floats in one visibility for one polarisation (real and imaginary)
const FLOATS_PER_COMPLEX: usize = 2;

/// Number of bytes in one f32
const BYTES_PER_FLOAT: usize = std::mem::size_of::<f32>();

/// Number of samples criterion takes for each benchmark
const SAMPLE_SIZE: usize = 10;

/// Makes an input buffer of `num_floats` floats where every value is different, so a wrong index would give a wrong
/// result (the reference tests check this; here it only makes the data realistic for the cache).
///
/// # Arguments
///
/// * `num_floats` - Number of floats in the buffer.
///
/// # Returns
///
/// * The buffer.
///
fn make_input(num_floats: usize) -> Vec<f32> {
    (0..num_floats).map(|i| i as f32).collect()
}

/// Gets the number of floats in one timestep and coarse channel of visibilities for a metafits.
///
/// # Arguments
///
/// * `context` - The metafits context.
///
/// # Returns
///
/// * The number of floats.
///
fn num_floats(context: &MetafitsContext) -> usize {
    context.num_baselines
        * context.num_corr_fine_chans_per_coarse
        * context.num_visibility_pols
        * FLOATS_PER_COMPLEX
}

fn bench_convert(c: &mut Criterion) {
    let legacy_context = MetafitsContext::new(LEGACY_METAFITS, Some(MWAVersion::CorrLegacy))
        .expect("Error creating legacy metafits context");
    // The MWAX test metafits is guessed as legacy if no version is given, so give it
    let mwax_context = MetafitsContext::new(MWAX_METAFITS, Some(MWAVersion::CorrMWAXv2))
        .expect("Error creating MWAX metafits context");

    let legacy_table = LegacyConversionTable::new(&legacy_context.rf_inputs);
    let legacy_num_floats = num_floats(&legacy_context);
    let legacy_input = make_input(legacy_num_floats);
    let mut legacy_output = vec![0.0; legacy_num_floats];

    let mwax_num_floats = num_floats(&mwax_context);
    let mwax_input = make_input(mwax_num_floats);
    let mut mwax_output = vec![0.0; mwax_num_floats];

    let mut group = c.benchmark_group("convert");
    group.sampling_mode(SamplingMode::Flat);

    // Report the number of bytes of visibilities that are converted per second
    group.throughput(Throughput::Bytes(
        (mwax_num_floats * BYTES_PER_FLOAT) as u64,
    ));

    group.bench_function("mwax_to_frequency_order", |b| {
        b.iter(|| {
            convert_mwax_hdu_to_frequency_order(
                black_box(&mwax_input),
                &mut mwax_output,
                mwax_context.num_baselines,
                mwax_context.num_corr_fine_chans_per_coarse,
                mwax_context.num_visibility_pols,
            );
            black_box(&mwax_output);
        })
    });

    group.bench_function("reference/copy", |b| {
        b.iter(|| {
            mwax_output.copy_from_slice(black_box(&mwax_input));
            black_box(&mwax_output);
        })
    });

    group.bench_function("reference/alloc_and_copy", |b| {
        b.iter(|| {
            let mut new_buffer: Vec<f32> = vec![0.0; mwax_num_floats];
            new_buffer.copy_from_slice(black_box(&mwax_input));
            black_box(new_buffer);
        })
    });

    group.throughput(Throughput::Bytes(
        (legacy_num_floats * BYTES_PER_FLOAT) as u64,
    ));

    group.bench_function("legacy_to_baseline_order", |b| {
        b.iter(|| {
            convert_legacy_hdu_to_mwax_baseline_order(
                &legacy_table,
                black_box(&legacy_input),
                &mut legacy_output,
                legacy_context.num_corr_fine_chans_per_coarse,
            );
            black_box(&legacy_output);
        })
    });

    group.bench_function("legacy_to_frequency_order", |b| {
        b.iter(|| {
            convert_legacy_hdu_to_mwax_frequency_order(
                &legacy_table,
                black_box(&legacy_input),
                &mut legacy_output,
                legacy_context.num_corr_fine_chans_per_coarse,
            );
            black_box(&legacy_output);
        })
    });

    group.finish();
}

criterion_group!(
    name = convert_benches;
    config = Criterion::default().sample_size(SAMPLE_SIZE).with_plots();
    targets = bench_convert
);
criterion_main!(convert_benches);
