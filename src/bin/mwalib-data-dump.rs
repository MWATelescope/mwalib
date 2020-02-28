/// Given gpubox files, provide a way to output/dump visibilities.
use anyhow::*;
use mwalib::*;
use std::fs::File;
use std::io::Write;
use structopt::StructOpt;

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

    /// Coarse channel
    #[structopt(long)]
    coarse_channel: usize,

    /// Path to the metafits file.
    #[structopt(short, long)]
    metafits: String,

    /// Paths to the gpubox files.
    #[structopt(name = "GPUBOX FILE")]
    files: Vec<String>,

    // Dump filename
    #[structopt(short, long)]
    dump_filename: String,
}

fn dump_data(
    metafits: String,
    files: Vec<String>,
    timestep: usize,
    baseline: usize,
    fine_channel_range: (usize, usize),
    coarse_channel: usize,
    dump_filename: String,
) -> Result<(), anyhow::Error> {
    let mut dump_file = File::create(dump_filename)?;
    println!("Dumping data via mwalib...");
    let mut context = mwalibContext::new(&metafits, &files)?;
    let coarse_channel_array = context.coarse_channels.clone();
    let timestep_array = context.timesteps.clone();

    println!("Correlator version: {}", context.corr_version);

    let floats_per_finechan = context.num_visibility_pols * 2;
    let floats_per_baseline = context.num_fine_channels * floats_per_finechan;

    let (ant1, ant2) = misc::get_antennas_from_baseline(baseline, context.num_antennas).unwrap();
    let ant1_name: String = context.antennas[ant1].tile_name.to_string();
    let ant2_name: String = context.antennas[ant2].tile_name.to_string();

    let baseline_index = baseline * floats_per_baseline;

    let ch1 = fine_channel_range.0;
    let ch2 = fine_channel_range.1;

    let ch_start_index = baseline_index + (ch1 * floats_per_finechan);
    let ch_end_index = baseline_index + (ch2 * floats_per_finechan) + floats_per_finechan;

    let mut sum: f64 = 0.;
    let mut float_count: u64 = 0;
    println!(
        "Dumping t={} coarse chan: {} ({}) {:.3} Mhz, fine ch: {}-{}, ant {} vs {}",
        timestep,
        coarse_channel,
        coarse_channel_array[coarse_channel].receiver_channel_number,
        (coarse_channel_array[coarse_channel].channel_centre_hz as f32 / 1.0e6),
        ch1,
        ch2,
        ant1_name,
        ant2_name
    );
    for (t, _) in timestep_array.iter().enumerate() {
        if t == timestep {
            println!("timestep: {}", t);
            for (c, _) in coarse_channel_array.iter().enumerate() {
                if c == coarse_channel {
                    println!("Reading timestep {}, coarse channel {}...", t, c);
                    let data =
                        context.read_one_timestep_coarse_channel_bfp(timestep, coarse_channel)?;
                    let mut fine_channel_counter = 0;
                    for v in (0..data.len()).step_by(floats_per_finechan) {
                        if v >= ch_start_index && v < ch_end_index {
                            writeln!(
                                &mut dump_file,
                                "{},{},{},{},{},{},{},{},{}",
                                ch1 + fine_channel_counter,
                                data[v],
                                data[v + 1],
                                data[v + 2],
                                data[v + 3],
                                data[v + 4],
                                data[v + 5],
                                data[v + 6],
                                data[v + 7],
                            )?;

                            sum = sum
                                + (data[v] as f64)
                                + (data[v + 1] as f64)
                                + (data[v + 2] as f64)
                                + (data[v + 3] as f64)
                                + (data[v + 4] as f64)
                                + (data[v + 5] as f64)
                                + (data[v + 6] as f64)
                                + (data[v + 7] as f64);
                            float_count += 8;

                            fine_channel_counter += 1;
                        }
                    }
                }
            }
        }
    }

    println!("Sum was {}, count was {} floats", sum, float_count);

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
        opts.coarse_channel,
        opts.dump_filename,
    )?;
    Ok(())
}
