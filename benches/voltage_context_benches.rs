use criterion::{criterion_group, criterion_main, Criterion, SamplingMode, Throughput};
use mwalib::{misc::generate_test_voltage_file, MWAVersion, VoltageContext};
use std::hint::black_box;
use std::path::Path;

/// The metafits of the MWAX VCS benchmark observation (144 tiles, so 288 rf inputs)
const MWAXVCS_METAFITS: &str = "test_files/1363085416/1363085416_metafits.fits";

/// The one voltage file the benchmarks read (8 seconds of 1 coarse channel). It is made on the first run and kept: it
/// is about 5.9 GB (160 voltage blocks of 64,000 samples x 288 rf inputs x 2 bytes, plus the header and delay block).
const MWAXVCS_FILE: &str = "test_files/1363085416/1363085416_1363085416_171.sub";

/// Values for `generate_test_voltage_file` that match the metafits
const MWAXVCS_NUM_VOLTAGE_BLOCKS: usize = 160;
const MWAXVCS_SAMPLES_PER_BLOCK: usize = 64_000;
const MWAXVCS_NUM_RF_INPUTS: usize = 288;
const MWAXVCS_NUM_FINE_CHANS: usize = 1;
const MWAXVCS_BYTES_PER_SAMPLE: usize = 2;
const MWAXVCS_INITIAL_VALUE: u8 = 0;

/// Number of seconds in an MWAX VCS file
const MWAXVCS_SECONDS_PER_FILE: usize = 8;

/// Number of seconds read by the small `read_second` benchmark
const MWAXVCS_SHORT_READ_SECONDS: usize = 1;

/// Make the benchmark voltage file if it does not exist yet.
fn setup_mwaxvcs_file() {
    if !Path::new(MWAXVCS_FILE).exists() {
        generate_test_voltage_file(
            MWAXVCS_FILE,
            MWAVersion::VCSMWAXv2,
            MWAXVCS_NUM_VOLTAGE_BLOCKS,
            MWAXVCS_SAMPLES_PER_BLOCK,
            MWAXVCS_NUM_RF_INPUTS,
            MWAXVCS_NUM_FINE_CHANS,
            MWAXVCS_BYTES_PER_SAMPLE,
            MWAXVCS_INITIAL_VALUE,
        )
        .expect("Error generating test voltage file");
    }
}

/// Benchmarks of reading MWAX VCS data.
///
/// Notes on what these measure:
/// * The file is read over and over, so after the first read it is in the operating system's file cache (if there is
///   enough memory for 5.9 GB). These benchmarks measure the copy from the file cache into our buffer, not the disk.
/// * The buffer is made once, outside the timed code, so no allocation is measured.
/// * Throughput is reported in bytes of voltages read per second.
fn bench_mwaxvcs_reads(c: &mut Criterion) {
    setup_mwaxvcs_file();

    let context = VoltageContext::new(MWAXVCS_METAFITS, &[MWAXVCS_FILE])
        .expect("Error creating voltage context");

    let timestep_index: usize = context.provided_timestep_indices[0];
    let gps_start: u64 = context.timesteps[timestep_index].gps_time_ms / 1000;
    let chan_index: usize = context.provided_coarse_chan_indices[0];
    let bytes_per_second =
        context.voltage_block_size_bytes as usize * context.num_voltage_blocks_per_second;

    let mut buffer: Vec<i8> = vec![0; bytes_per_second * MWAXVCS_SECONDS_PER_FILE];

    let mut group = c.benchmark_group("mwaxvcs");
    group.sampling_mode(SamplingMode::Flat);

    group.throughput(Throughput::Bytes(buffer.len() as u64));

    group.bench_function("read_file", |b| {
        b.iter(|| {
            context
                .read_file(timestep_index, chan_index, &mut buffer)
                .expect("Error calling read_file");
            black_box(&buffer);
        })
    });

    group.bench_function("read_second_8s", |b| {
        b.iter(|| {
            context
                .read_second(gps_start, MWAXVCS_SECONDS_PER_FILE, chan_index, &mut buffer)
                .expect("Error calling read_second");
            black_box(&buffer);
        })
    });

    // A smaller read (1 second, about 738 MB), which needs less memory for the file cache
    let short_len = bytes_per_second * MWAXVCS_SHORT_READ_SECONDS;
    group.throughput(Throughput::Bytes(short_len as u64));

    group.bench_function("read_second_1s", |b| {
        b.iter(|| {
            context
                .read_second(
                    gps_start,
                    MWAXVCS_SHORT_READ_SECONDS,
                    chan_index,
                    &mut buffer[..short_len],
                )
                .expect("Error calling read_second");
            black_box(&buffer);
        })
    });

    group.finish();
}

criterion_group!(
    name = voltage_context_benches;
    config = Criterion::default().sample_size(10).with_plots();
    targets = bench_mwaxvcs_reads
);
criterion_main!(voltage_context_benches);
