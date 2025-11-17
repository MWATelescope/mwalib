use criterion::{criterion_group, criterion_main, Criterion};
use mwalib::{misc::generate_test_voltage_file, MWAVersion, VoltageContext};
use std::path::Path;

fn setup_test_files_1363085416() -> (String, Vec<String>) {
    let metafits_filename = String::from("test_files/1363085416/1363085416_metafits.fits");

    let filenames: Vec<String> = [
        String::from("test_files/1363085416/1363085416_1363085416_171.sub"),
        String::from("test_files/1363085416/1363085416_1363085424_171.sub"),
        String::from("test_files/1363085416/1363085416_1363085432_171.sub"),
        String::from("test_files/1363085416/1363085416_1363085440_171.sub"),
    ]
    .to_vec();

    // Don't recreate the files if they exist
    for filename in &filenames {
        if !Path::new(&filename).exists() {
            generate_test_voltage_file(&filename, MWAVersion::VCSMWAXv2, 160, 64_000, 288, 1, 2, 0)
                .expect("Error generating test voltage file");
        }
    }

    return (metafits_filename, filenames);
}

fn bench_mwaxvcs_read_second(c: &mut Criterion) {
    let (metafits_filename, filenames) = setup_test_files_1363085416();

    // Open a context and load in a test metafits and gpubox file
    let context = VoltageContext::new(&metafits_filename, &filenames)
        .expect("Error creating voltage context");

    //
    // Now do a read of the data from time 0, channel 0
    //
    let gps_start: u64 = context.timesteps[context.provided_timestep_indices[0]].gps_time_ms / 1000; // Get first provided timestep
    let gps_count: usize = context.provided_timestep_indices.len(); // Get last provided timestep
    let chan_index: usize = context.provided_coarse_chan_indices[0]; // Get first provided cc

    // Create output buffer
    let mut buffer: Vec<i8> = vec![
        0;
        context.voltage_block_size_bytes as usize
            * context.num_voltage_blocks_per_second
            * gps_count
    ];

    // Bench this!
    c.bench_function("mwaxvcs_read_second", |b| {
        b.iter(|| {
            context
                .read_second(gps_start, gps_count, chan_index, &mut buffer)
                .expect("Error calling read_second");
        })
    });

    c.bench_function("mwaxvcs_read_second2", |b| {
        b.iter(|| {
            context
                .read_second2(gps_start, gps_count, chan_index, &mut buffer)
                .expect("Error calling read_second2");
        })
    });
}

criterion_group!(
    name = voltage_context_benches;
    config = Criterion::default().sample_size(10).with_plots();
    targets = bench_mwaxvcs_read_second

);
criterion_main!(voltage_context_benches);
