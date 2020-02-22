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

    /// Fine channel to start with
    #[structopt(long)]
    fine_chan1: usize,
    /// Fine channel to end with
    #[structopt(long)]
    fine_chan2: usize,

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
    fine_channel_range: (usize, usize),
) -> Result<(), anyhow::Error> {
    println!("Dumping data via mwalib...");
    let mut context = mwalibContext::new(&metafits, &files)?;
    context.num_data_scans = context.num_timesteps;
    println!("Correlator version: {}", context.corr_version);

    let floats_per_finechan = context.num_visibility_pols * 2;
    let floats_per_baseline = context.num_fine_channels * floats_per_finechan;

    let (ant1, ant2) = misc::get_antennas_from_baseline(baseline, context.num_antennas).unwrap();
    let ant1_name: String = context.rf_inputs[ant1 * 2].tile_name.to_string();
    let ant2_name: String = context.rf_inputs[ant2 * 2].tile_name.to_string();

    let baseline_index = baseline * floats_per_baseline;

    let ch1 = fine_channel_range.0;
    let ch2 = fine_channel_range.1;

    let ch_start_index = baseline_index + (ch1 * floats_per_finechan);
    let ch_end_index = baseline_index + (ch2 * floats_per_finechan) + floats_per_finechan;

    println!(
        "Dumping t={} ch: {}-{} ant {} vs {}",
        timestep, ch1, ch2, ant1_name, ant2_name
    );

    let data = &context.read(context.num_data_scans)?;

    // 53, 2, lots
    println!("{} {} {}", data.len(), data[0].len(), data[0][0].len());

    for (t, time) in data.iter().enumerate() {
        if t == timestep {
            println!("timestep: {}", t);
            for (c, channel) in time.iter().enumerate() {
                println!(
                    "Coarse channel: {:.3} MHz",
                    (context.coarse_channels[c].channel_centre_hz as f32 / 1_000_000.)
                );
                let mut fine_channel_counter = 0;
                for v in (0..channel.len()).step_by(floats_per_finechan) {
                    if v >= ch_start_index && v < ch_end_index {
                        println!(
                        "ch{:3} {:>10.2},{:>10.2} | {:>10.2},{:>10.2} | {:>10.2},{:>10.2} | {:>10.2},{:>10.2}",
                        ch1 + fine_channel_counter,
                        channel[v],
                        channel[v + 1],
                        channel[v + 2],
                        channel[v + 3],
                        channel[v + 4],
                        channel[v + 5],
                        channel[v + 6],
                        channel[v + 7]
                        );

                        fine_channel_counter += 1;
                    }
                }
            }
        }
    }

    Ok(())
}

fn main() -> Result<(), anyhow::Error> {
    let opts = Opt::from_args();

    dump_data(
        opts.metafits,
        opts.files,
        opts.timestep,
        opts.baseline,
        (opts.fine_chan1, opts.fine_chan2),
    )?;
    Ok(())
}
