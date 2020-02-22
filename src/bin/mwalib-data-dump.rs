/// Given gpubox files, provide a way to output/dump visibilities.
use anyhow::*;
use structopt::StructOpt;

use mwalib::*;

#[derive(StructOpt, Debug)]
#[structopt(name = "mwalib-data-dump", author)]
struct Opt {
    /// timestep number (0 indexed)
    #[structopt(short, long)]
    timestep: usize,

    /// baseline number (0 indexed)
    #[structopt(short, long)]
    baseline: usize,

    /// Path to the metafits file.
    #[structopt(short, long)]
    metafits: String,

    /// Paths to the gpubox files.
    #[structopt(name = "GPUBOX FILE")]
    files: Vec<String>,
}

fn dump_data(
    metafits: String,
    files: Vec<String>,
    timestep: usize,
    baseline: usize,
) -> Result<(), anyhow::Error> {
    println!("Dumping data via mwalib...");
    let mut context = mwalibContext::new(&metafits, &files)?;
    context.num_data_scans = 1;
    println!("Correlator version: {}", context.corr_version);

    let floats_per_finechan = context.num_visibility_pols * 2;
    let floats_per_baseline = context.num_fine_channels * floats_per_finechan;

    let (ant1, ant2) = misc::get_antennas_from_baseline(baseline, context.num_antennas).unwrap();
    let ant1_name: String = context.rf_inputs[ant1 * 2].tile_name.to_string();
    let ant2_name: String = context.rf_inputs[ant2 * 2].tile_name.to_string();

    let baseline_index = baseline * floats_per_baseline;

    let ch1 = 0;
    let ch2 = 127;

    let ch_start_index = baseline_index + (ch1 * floats_per_finechan);
    let ch_end_index = baseline_index + (ch2 * floats_per_finechan) + floats_per_finechan;

    println!(
        "Dumping t={} ch: {}-{} ant {} vs {}",
        timestep, ch1, ch2, ant1_name, ant2_name
    );

    let mut current_timestep = 0;
    while context.num_data_scans != 0 {
        let slice = &context.read(context.num_data_scans)?[0][0][ch_start_index..ch_end_index];

        if current_timestep == timestep {
            for v in (0..slice.len()).step_by(floats_per_finechan) {
                println!(
                    "ch{:3} {:>10.2},{:>10.2} | {:>10.2},{:>10.2} | {:>10.2},{:>10.2} | {:>10.2},{:>10.2}",
                    ch1 + (v / floats_per_finechan),
                    slice[v],
                    slice[v + 1],
                    slice[v + 2],
                    slice[v + 3],
                    slice[v + 4],
                    slice[v + 5],
                    slice[v + 6],
                    slice[v + 7]
                );
            }
            break;
        } else {
            current_timestep += 1;
        }
    }
    Ok(())
}

fn main() -> Result<(), anyhow::Error> {
    let opts = Opt::from_args();

    dump_data(opts.metafits, opts.files, opts.timestep, opts.baseline)?;
    Ok(())
}
