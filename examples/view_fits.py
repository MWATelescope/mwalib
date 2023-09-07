import argparse
import matplotlib
import matplotlib.pyplot as plt
import numpy as np
import math
from pymwalib.common import MWAVersion
import pymwalib.correlator_context

# Allow X windows support- but you need to
# pip install pycairo
# pip install pygobject
# first.
matplotlib.use("GTK3Agg")

dpi = 100
MODE_RANGE = "RANGE"
MODE_BASELINE = "BASELINE"


class ViewFITSArgs:
    def __init__(self, passed_args):
        self.filename = passed_args["filename"]
        self.metafits_filename = passed_args["metafits"]

        self.time_step1 = passed_args["timestep1"]
        self.time_step2 = passed_args["timestep2"]

        self.tile1 = passed_args["ant1"]
        self.tile2 = passed_args["ant2"]

        self.autos_only = passed_args["autosonly"]

        self.channel1 = passed_args["channel1"]
        self.channel2 = passed_args["channel2"]

        self.ppd_plot = passed_args["ppdplot"]
        self.ppd_plot2 = passed_args["ppdplot2"]
        self.grid_plot = passed_args["gridplot"]
        self.grid_plot2 = passed_args["gridplot2"]
        self.grid_pol = passed_args["gridpol"]
        self.phase_plot_all = passed_args["phaseplot_all"]
        self.phase_plot_one = passed_args["phaseplot_one"]
        self.mode = passed_args["mode"]

        self.dumpraw = passed_args["dump_raw"]
        self.dumpplot = passed_args["dump_plot"]

        # Are we plotting?
        self.any_plotting = (
            self.ppd_plot
            or self.ppd_plot2
            or self.grid_plot
            or self.phase_plot_all
            or self.phase_plot_one
        )

        if not self.any_plotting and self.dumpplot:
            print("You must be plotting to dump out the plot data to file!")
            exit(-1)

        # Some constants not found in fits file
        self.values = 2  # real and imaginary

        self.validate_params()

    def validate_params(self):  # noqa: C901
        # Read fits file
        print(
            "Opening with pymwalib using metafits file "
            f"{self.metafits_filename} and data file {self.filename}..."
        )
        self.context = pymwalib.correlator_context.CorrelatorContext(
            self.metafits_filename,
            [
                self.filename,
            ],
        )

        for p in self.context.provided_coarse_chan_indices:
            print(self.context.coarse_channels[p])

        self.correlator_version: pymwalib.common.MWAVersion = (
            self.context.mwa_version
        )
        self.pols = (
            self.context.metafits_context.num_visibility_pols
        )  # xx,xy,yx,yy

        # in v2 NAXIS1 == fine channels * pols * r,i
        # in v2 NAXIS2 == baselines
        # Get number of tiles based on the number of signal chains
        self.fits_tiles = len(self.context.metafits_context.antennas)
        self.chan_x_pols_x_vals = (
            self.context.metafits_context.num_corr_fine_chans_per_coarse
            * self.pols
            * self.values
        )
        self.fits_channels = (
            self.context.metafits_context.num_corr_fine_chans_per_coarse
        )
        self.fits_has_weights = True

        # Check mode
        if self.mode != MODE_RANGE and self.mode != MODE_BASELINE:
            print(f"Error, mode should be {MODE_RANGE} or {MODE_BASELINE}!")
            exit(-1)

        # Check tiles
        if self.tile1 == -1:
            self.tile1 = 0

        if self.tile2 == -1:
            self.tile2 = self.fits_tiles - 1

        if self.tile1 > self.fits_tiles - 1:
            print("Error ant1 is more than the last tile index!")
            exit(-1)

        if self.tile2 > self.fits_tiles - 1:
            print("Error ant2 is more than the last tile index!")
            exit(-1)

        if self.tile1 > self.tile2:
            print("Error ant1 is more than the ant2!")
            exit(-1)

        # Check time steps
        self.fits_time_steps = self.context.num_timesteps

        if self.time_step1 == -1:
            self.time_step1 = 1

        if self.time_step2 == -1:
            self.time_step2 = self.fits_time_steps

        if self.time_step1 > self.fits_time_steps:
            print("Error t1 is more than the max time step!")
            exit(-1)

        if self.time_step2 > self.fits_time_steps:
            print("Error t2 is more than the max time step!")
            exit(-1)

        if self.time_step1 > self.time_step2:
            print("Error t1 is more than the t2!")
            exit(-1)

        # Check channels
        if self.channel1 == -1:
            self.channel1 = 0

        if self.channel2 == -1:
            self.channel2 = self.fits_channels - 1

        if self.channel1 > self.fits_channels - 1:
            print("Error c1 is more than the number of the last fine channel!")
            exit(-1)

        if self.channel2 > self.fits_channels - 1:
            print("Error c2 is more than the number of the last fine channel!")
            exit(-1)

        if self.channel1 > self.channel2:
            print("Error c1 is more than the c2!")
            exit(-1)

        # Some calculated fields
        self.tile_count = self.tile2 - self.tile1 + 1

        if self.mode == MODE_BASELINE:
            self.baseline_count = 1
        else:
            self.baseline_count = int(
                (self.tile_count * (self.tile_count + 1)) / 2
            )

        self.fits_baseline_count = int(
            (self.fits_tiles * (self.fits_tiles + 1)) / 2
        )
        self.channel_count = self.channel2 - self.channel1 + 1
        self.time_step_count = self.time_step2 - self.time_step1 + 1

        self.hdu_time1 = self.time_step1 - 1
        self.unix_time1 = (
            self.context.timesteps[self.hdu_time1].unix_time_ms / 1000.0
        )

        self.hdu_time2 = self.time_step2 - 1
        self.unix_time2 = (
            self.context.timesteps[self.hdu_time2].unix_time_ms / 1000.0
        )

        # print params
        if self.grid_plot2:
            self.param_string = (
                f"[{self.grid_pol}] {self.filename} "
                f"t={self.time_step1}-{self.time_step2} "
                f"t_hdu={self.hdu_time1}-{self.hdu_time2} \n"
                f"t_unix={self.unix_time1}-{self.unix_time2} "
                f"tile={self.tile1}-{self.tile2} "
                f"ch={self.channel1}-{self.channel2} "
                f"autosonly?={self.autos_only} "
                f"{self.tile_count}t/{self.fits_tiles}t"
            )
        else:
            self.param_string = (
                f"{self.filename} t={self.time_step1}-{self.time_step2} "
                f"t_hdu={self.hdu_time1}-{self.hdu_time2} \n"
                f"t_unix={self.unix_time1}-{self.unix_time2} "
                f"tile={self.tile1}-{self.tile2} "
                f"ch={self.channel1}-{self.channel2} "
                f"autosonly?={self.autos_only} "
                f"{self.tile_count}t/{self.fits_tiles}t"
            )
        print(self.param_string)


def meets_criteria(i, j, a1, a2, mode):
    # mode == RANGE | BASELINE
    if mode == MODE_RANGE:
        return i >= a1 and j <= a2
    elif mode == MODE_BASELINE:
        return i == a1 and j == a2
    else:
        return None


# v1 = baseline, freq, pol
# v2 = freq,baseline,pol
def peek_fits(program_args: ViewFITSArgs):  # noqa: C901
    print("Initialising data structures...")

    plot_ppd_data_x = None
    plot_ppd_data_y = None
    plot_ppd2_data_x = None
    plot_ppd2_data_y = None
    plot_grid_data = None
    plot_grid2_data = None
    plot_phase_data_x = None
    plot_phase_data_y = None

    # ppd array will be [timestep][channel]
    if program_args.ppd_plot:
        plot_ppd_data_x = np.empty(
            shape=(program_args.channel_count, program_args.time_step_count)
        )
        plot_ppd_data_x.fill(0)
        plot_ppd_data_y = np.empty(
            shape=(program_args.channel_count, program_args.time_step_count)
        )
        plot_ppd_data_y.fill(0)

    # ppd plot 2 array will be [timestep][baseline][channel]
    if program_args.ppd_plot2:
        plot_ppd2_data_x = np.empty(
            shape=(
                program_args.time_step_count,
                program_args.baseline_count,
                program_args.channel_count,
            )
        )
        plot_ppd2_data_x.fill(0)
        plot_ppd2_data_y = np.empty(
            shape=(
                program_args.time_step_count,
                program_args.baseline_count,
                program_args.channel_count,
            )
        )
        plot_ppd2_data_y.fill(0)

    # Grid plot
    if program_args.grid_plot:
        plot_grid_data = np.empty(
            shape=(
                program_args.time_step_count,
                program_args.tile_count,
                program_args.tile_count,
            )
        )
        plot_grid_data.fill(0)

    # Grid plot 2 array
    if program_args.grid_plot2:
        plot_grid2_data = np.empty(
            shape=(
                program_args.time_step_count,
                program_args.tile_count,
                program_args.tile_count,
            )
        )
        plot_grid2_data.fill(0)

    # Phase plot
    if program_args.phase_plot_all or program_args.phase_plot_one:
        plot_phase_data_x = np.empty(
            shape=(
                program_args.time_step_count,
                program_args.baseline_count,
                program_args.channel_count,
            )
        )
        plot_phase_data_x.fill(0)
        plot_phase_data_y = np.empty(
            shape=(
                program_args.time_step_count,
                program_args.baseline_count,
                program_args.channel_count,
            )
        )
        plot_phase_data_y.fill(0)

    raw_dump_file = None
    plot_dump_file = None

    if program_args.correlator_version == MWAVersion.CorrMWAXv2.value:
        filename = f"{program_args.context.metafits_context.obs_id}_mwax.csv"
    else:
        filename = f"{program_args.context.metafits_context.obs_id}_mwa.csv"

    # Open a file for dumping the plot values
    if program_args.dumpplot:
        plot_dump_filename = f"plot_dump_{filename}"
        print(f"Openning {plot_dump_filename} for writing")
        plot_dump_file = open(plot_dump_filename, "w")

        # Write the csv header depending on the type of plot
        if program_args.ppd_plot:
            plot_dump_file.write("time_index, fine_chan, x, y\n")

        elif program_args.ppd_plot2:
            plot_dump_file.write("plot_number, time_index, fine_chan, x, y\n")

        elif program_args.grid_plot:
            plot_dump_file.write(
                "unix_time, tile1, tile2, log10_scaled_power\n"
            )

        elif program_args.grid_plot2:
            plot_dump_file.write(
                "unix_time, tile1, tile2, log10_scaled_power\n"
            )

        elif program_args.phase_plot_one:
            plot_dump_file.write("time_index, baseline, x, y\n")

        elif program_args.phase_plot_all:
            plot_dump_file.write("time_index, baseline, x, y\n")

    # print a csv header for the raw dump
    if program_args.dumpraw:
        raw_dump_filename = f"raw_dump_{filename}"
        print(f"Openning {raw_dump_filename} for writing")
        raw_dump_file = open(raw_dump_filename, "w")

        raw_dump_file.write(
            "unix_time, baseline, ant1, ant2, fine_ch, "
            "xx_r, xx_i, xy_r, xy_i, yx_r, yx_i, yy_r, yy_i, "
            "xx_pow, xy_pow, yx_pow, yy_pow, x_phase_deg, y_phase_deg\n"
        )

    for timestep_index, timestep in enumerate(program_args.context.timesteps):
        time_index = timestep_index - program_args.hdu_time1

        if (
            timestep_index < program_args.hdu_time1
            or timestep_index > program_args.hdu_time2
        ):
            # print(f"Skipping timestep index {timestep_index} (out of range)")
            continue
        else:
            print(
                f"Processing timestep: {timestep.index} "
                f"(time index: {time_index})..."
            )

        # Read data
        data = program_args.context.read_by_baseline(
            timestep.index,
            program_args.context.provided_coarse_chan_indices[0],
        )
        data = data.reshape(
            program_args.context.metafits_context.num_baselines,
            program_args.chan_x_pols_x_vals,
        )

        baseline = 0
        selected_baseline = 0

        # Print all tile info but only for the first timestep we have
        if time_index == 0:
            print(
                f"QUAKTIME:"
                f"{program_args.context.metafits_context.quack_time_duration_ms/1000.} s\n"
            )

            print("\nUnflagged tiles:")
            print("================")
            for i in range(
                0, len(program_args.context.metafits_context.antennas)
            ):
                if (
                    program_args.context.metafits_context.antennas[
                        i
                    ].rf_input_x.flagged
                    is False
                    and program_args.context.metafits_context.antennas[
                        i
                    ].rf_input_y.flagged
                    is False
                ):
                    print(
                        f"Index {i}, TileID: {program_args.context.metafits_context.antennas[i].tile_id} "
                        f"{program_args.context.metafits_context.antennas[i].tile_name}"
                        f" (rec:{program_args.context.metafits_context.antennas[i].rf_input_x.rec_number},"
                        f"slot:{program_args.context.metafits_context.antennas[i].rf_input_x.rec_slot_number})"
                    )

            print("\nFlagged tiles:")
            print("================")
            for i in range(
                0, len(program_args.context.metafits_context.antennas)
            ):
                if (
                    program_args.context.metafits_context.antennas[
                        i
                    ].rf_input_x.flagged
                    or program_args.context.metafits_context.antennas[
                        i
                    ].rf_input_y.flagged
                ):
                    print(
                        f"Index {i}, TileID: {program_args.context.metafits_context.antennas[i].tile_id} "
                        f"{program_args.context.metafits_context.antennas[i].tile_name}"
                        f" (rec:{program_args.context.metafits_context.antennas[i].rf_input_x.rec_number},"
                        f"slot:{program_args.context.metafits_context.antennas[i].rf_input_x.rec_slot_number})"
                    )

        for i in range(0, program_args.tile2 + 1):
            for j in range(i, program_args.fits_tiles):
                # Explaining this if:
                # Line 1. Check for autos if that's what we asked for
                # Line 2. OR just be True if we didn't ask for autos only.
                # Line 3 (Applicable to cases 1 and 2 above):
                #         Check the selected tile1 and tile2 are in range
                if (
                    (i == j and program_args.autos_only is True)
                    or (program_args.autos_only is False)
                ) and (
                    meets_criteria(
                        i,
                        j,
                        program_args.tile1,
                        program_args.tile2,
                        program_args.mode,
                    )
                ):

                    for chan in range(
                        program_args.channel1, program_args.channel2 + 1
                    ):
                        index = chan * (
                            program_args.pols * program_args.values
                        )

                        if (
                            program_args.context.metafits_context.antennas[
                                i
                            ].rf_input_x.flagged
                            or program_args.context.metafits_context.antennas[
                                i
                            ].rf_input_y.flagged
                            or program_args.context.metafits_context.antennas[
                                j
                            ].rf_input_x.flagged
                            or program_args.context.metafits_context.antennas[
                                j
                            ].rf_input_y.flagged
                        ) and 1 == 0:
                            xx_r = 0
                            xx_i = 0

                            xy_r = 0
                            xy_i = 0

                            yx_r = 0
                            yx_i = 0

                            yy_r = 0
                            yy_i = 0

                            power_xx = 0
                            power_xy = 0
                            power_yx = 0
                            power_yy = 0

                            phase_x = 0
                            phase_y = 0

                        else:
                            xx_r = data[baseline][index]
                            xx_i = data[baseline][index + 1]

                            xy_r = data[baseline][index + 2]
                            xy_i = data[baseline][index + 3]

                            yx_r = data[baseline][index + 4]
                            yx_i = data[baseline][index + 5]

                            yy_r = data[baseline][index + 6]
                            yy_i = data[baseline][index + 7]

                            power_xx = xx_i * xx_i + xx_r * xx_r
                            power_xy = xy_i * xy_i + xy_r * xy_r
                            power_yx = yx_i * yx_i + yx_r * yx_r
                            power_yy = yy_i * yy_i + yy_r * yy_r

                            phase_x = math.degrees(math.atan2(xx_i, xx_r))
                            phase_y = math.degrees(math.atan2(yy_i, yy_r))

                        if program_args.ppd_plot:
                            plot_ppd_data_x[chan][
                                time_index
                            ] = plot_ppd_data_x[chan][time_index] + (
                                power_xx / program_args.baseline_count
                            )
                            plot_ppd_data_y[chan][
                                time_index
                            ] = plot_ppd_data_y[chan][time_index] + (
                                power_yy / program_args.baseline_count
                            )  # noqa: E127

                            # No idea, but the line above fails E127 in PEP8.
                            # I have no idea why. The line above it
                            # does not fail... weird

                        elif program_args.ppd_plot2:
                            plot_ppd2_data_x[time_index][selected_baseline][
                                chan
                            ] = power_xx
                            plot_ppd2_data_y[time_index][selected_baseline][
                                chan
                            ] = power_yy

                        elif program_args.grid_plot:
                            plot_grid_data[time_index][j][i] = (
                                plot_grid_data[time_index][j][i]
                                + power_xx
                                + power_yy
                            )

                        elif program_args.grid_plot2:
                            # offset index by the polarisation
                            if program_args.grid_pol == "XX":
                                plot_grid2_data[time_index][j][i] += power_xx
                            elif program_args.grid_pol == "XY":
                                plot_grid2_data[time_index][j][i] += power_xy
                            elif program_args.grid_pol == "YX":
                                plot_grid2_data[time_index][j][i] += power_yx
                            elif program_args.grid_pol == "YY":
                                plot_grid2_data[time_index][j][i] += power_yy
                            else:
                                print(
                                    "Grid Plot requires you to specify "
                                    "-gp (--gridpol) and "
                                    "takes XX,XY,YX,YY as parameters"
                                )
                                exit(-1)

                        elif (
                            program_args.phase_plot_all
                            or program_args.phase_plot_one
                        ):
                            plot_phase_data_x[time_index][selected_baseline][
                                chan
                            ] = phase_x
                            plot_phase_data_y[time_index][selected_baseline][
                                chan
                            ] = phase_y

                        if program_args.dumpraw:
                            raw_dump_file.write(
                                f"{timestep.unix_time_ms / 1000.0},"
                                f"{baseline},"
                                f"{i},"
                                f"{j},"
                                f"{chan},"
                                f"{xx_r},"
                                f"{xx_i},"
                                f"{xy_r},"
                                f"{xy_i},"
                                f"{yx_r},"
                                f"{yx_i},"
                                f"{yy_r},"
                                f"{yy_i},"
                                f"{power_xx},"
                                f"{power_xy},"
                                f"{power_yx},"
                                f"{power_yy},"
                                f"{phase_x},"
                                f"{phase_y}\n"
                            )

                    selected_baseline = selected_baseline + 1

                baseline = baseline + 1

    print("Processing of data done!")

    if program_args.dumpraw:
        if raw_dump_file:
            raw_dump_file.close()

    if program_args.ppd_plot:
        convert_to_db = False
        do_ppd_plot(
            program_args.param_string,
            program_args,
            plot_ppd_data_x,
            plot_ppd_data_y,
            convert_to_db,
        )

    if program_args.ppd_plot2:
        convert_to_db = False
        do_ppd_plot2(
            program_args.param_string,
            program_args,
            plot_ppd2_data_x,
            plot_ppd2_data_y,
            convert_to_db,
        )

    if program_args.grid_plot:
        do_grid_plot(program_args.param_string, program_args, plot_grid_data)

    if program_args.grid_plot2:
        do_grid_plot2(program_args.param_string, program_args, plot_grid2_data)

    if program_args.phase_plot_all or program_args.phase_plot_one:
        do_phase_plot(
            program_args.param_string,
            program_args,
            plot_phase_data_x,
            plot_phase_data_y,
        )

    if program_args.dumpplot:
        if plot_dump_file:
            plot_dump_file.close()

    print("view_fits Done!\n")


def do_ppd_plot(
    title,
    program_args: ViewFITSArgs,
    plot_ppd_data_x,
    plot_ppd_data_y,
    convert_to_db,
):
    # This will sum across timesteps and baselines to make a single ppd
    print("Preparing ppd plot...")

    min_db = 0

    # Convert to a dB figure
    if convert_to_db:
        for t in range(0, program_args.time_step_count):
            for c in range(0, program_args.channel_count):
                plot_ppd_data_x[c][t] = (
                    math.log10(plot_ppd_data_x[c][t] + 1) * 10
                )
                plot_ppd_data_y[c][t] = (
                    math.log10(plot_ppd_data_y[c][t] + 1) * 10
                )

        # Get min dB value
        # Previously had min(np.array(plot_ppd_data_x).min(),
        #                    np.array(plot_ppd_data_y).min())
        min_db = 0

    fig, ax = plt.subplots(
        figsize=(20, 10),
        nrows=1,
        ncols=1,
        squeeze=False,
        sharey="all",
        dpi=dpi,
    )
    fig.suptitle(title)

    for t in range(0, program_args.time_step_count):
        print(f"Adding data points for time ({t})...")

        # Step down the dB by the min so we have a 0 base
        for c in range(0, program_args.channel_count):
            plot_ppd_data_x[c][t] = plot_ppd_data_x[c][t] - min_db
            plot_ppd_data_y[c][t] = plot_ppd_data_y[c][t] - min_db

    # Get the current plot
    plot = ax[0][0]

    # Draw this plot
    plot.plot(plot_ppd_data_x, "o", color="blue")
    plot.plot(plot_ppd_data_y, "o", color="green")

    # Set labels
    if convert_to_db:
        plot.set_ylabel("dB", size=6)
    else:
        plot.set_ylabel("Raw value", size=6)

    plot.set_xlabel("fine channel", size=6)

    # Set plot title
    plot.set_title(f"t={program_args.unix_time1}", size=6)

    print("Saving figure...")

    # Save the final plot to disk
    if program_args.correlator_version == MWAVersion.CorrMWAXv2.value:
        filename = "ppd_plot_mwax.png"
    else:
        filename = "ppd_plot_mwa.png"
    plt.savefig(filename, bbox_inches="tight", dpi=dpi)
    print(f"saved {filename}")
    plt.show()


def do_ppd_plot2(
    title,
    program_args: ViewFITSArgs,
    plot_ppd_data_x,
    plot_ppd_data_y,
    convert_to_db,
):  # noqa: C901
    print("Preparing ppd plot2...")

    # Work out layout of plots
    plots = program_args.time_step_count
    plot_rows = math.floor(math.sqrt(plots))
    plot_cols = math.ceil(plots / plot_rows)
    plot_row = 0
    plot_col = 0
    min_db = 0

    # Convert to a dB figure
    if convert_to_db:
        for t in range(0, program_args.time_step_count):
            for c in range(0, program_args.channel_count):
                for b in range(0, program_args.baseline_count):
                    plot_ppd_data_x[t][b][c] = (
                        math.log10(plot_ppd_data_x[t][b][c] + 1) * 10
                    )
                    plot_ppd_data_y[t][b][c] = (
                        math.log10(plot_ppd_data_y[t][b][c] + 1) * 10
                    )

        # Get min dB value
        min_db = min(
            np.array(plot_ppd_data_x).min(initial=0),
            np.array(plot_ppd_data_y).min(initial=0),
        )

    fig, ax = plt.subplots(
        figsize=(20, 10),
        nrows=plot_rows,
        ncols=plot_cols,
        squeeze=False,
        sharey="all",
        dpi=dpi,
    )
    fig.suptitle(title)

    for t in range(0, program_args.time_step_count):
        # print(f"Adding data points for plot({t})...")

        # Step down the dB by the min so we have a 0 base
        if min_db != 0:
            for c in range(0, program_args.channel_count):
                for b in range(0, program_args.baseline_count):
                    plot_ppd_data_x[t][b][c] = (
                        plot_ppd_data_x[t][b][c] - min_db
                    )
                    plot_ppd_data_y[t][b][c] = (
                        plot_ppd_data_y[t][b][c] - min_db
                    )

        # Get the current plot
        plot = ax[plot_row][plot_col]

        # Draw this plot
        for b in range(0, program_args.baseline_count):
            plot.plot(plot_ppd_data_x[t][b], "o", markersize=1, color="blue")
            plot.plot(plot_ppd_data_y[t][b], "o", markersize=1, color="green")

        # Set labels
        if convert_to_db:
            plot.set_ylabel("dB", size=6)
        else:
            plot.set_ylabel("Raw value", size=6)

        plot.set_xlabel("fine channel", size=6)

        # Set plot title
        plot.set_title(f"t={t + program_args.time_step1}", size=6)

        # Increment so we know which plot we are on
        if plot_col < plot_cols - 1:
            plot_col = plot_col + 1
        else:
            plot_row = plot_row + 1
            plot_col = 0

    print("Saving figure...")

    # Save the final plot to disk
    if program_args.correlator_version == MWAVersion.CorrMWAXv2.value:
        filename = "ppd_plot2_mwax.png"
    else:
        filename = "ppd_plot2_mwa.png"
    plt.savefig(filename, bbox_inches="tight", dpi=dpi)
    print(f"saved {filename}")
    plt.show()


def do_grid_plot(
    title, program_args: ViewFITSArgs, plot_grid_data
):  # noqa: C901
    print("Preparing grid plot...")

    # Work out layout of plots
    plots = program_args.time_step_count
    plot_rows = math.floor(math.sqrt(plots))
    plot_cols = math.ceil(plots / plot_rows)
    plot_row = 0
    plot_col = 0

    if 1 == 1:
        for time_index in range(0, program_args.time_step_count):
            for t1 in range(0, program_args.tile_count):
                for t2 in range(t1, program_args.tile_count):
                    plot_grid_data[time_index][t2][t1] = (
                        math.log10(plot_grid_data[time_index][t2][t1] + 1) * 10
                    )

        # Get min dB value
        # np_array_nonzero = plot_grid_data[plot_grid_data > 1]
        # min_db = np_array_nonzero.min()

        # Apply min_db but only to values > 1
        # plot_grid_data = np.where(plot_grid_data > 1,
        # plot_grid_data - min_db, plot_grid_data)

    fig, ax = plt.subplots(
        figsize=(30, 30),
        nrows=plot_rows,
        ncols=plot_cols,
        squeeze=False,
        sharex="all",
        sharey="all",
        dpi=dpi,
    )
    fig.suptitle(title)

    n_step = math.ceil(plots / 1.25)

    if n_step % 2 != 0:
        n_step = n_step + 1

    if n_step % 4 != 0:
        n_step = n_step - 2

    if n_step <= 1:
        n_step = 1
    else:
        n_step = n_step * 2

    if n_step > 16:
        n_step = 16

    for time_index in range(0, program_args.time_step_count):
        for t1 in range(0, program_args.tile_count):
            for t2 in range(t1, program_args.tile_count):
                print(
                    time_index,
                    program_args.tile1 + t1,
                    program_args.tile1 + t2,
                    plot_grid_data[time_index][t2][t1],
                )

        print(f"Adding data points for plot({time_index})...")

        # Get the current plot
        plot = ax[plot_row][plot_col]

        plot.imshow(
            plot_grid_data[time_index], cmap="inferno", interpolation="None"
        )

        plot.set_title(f"t={time_index+program_args.time_step1}", size=6)
        plot.set_xticks(np.arange(program_args.tile_count, step=n_step))
        plot.set_yticks(np.arange(program_args.tile_count, step=n_step))
        plot.set_xticklabels(
            np.arange(program_args.tile1, program_args.tile2 + 1, step=n_step)
        )
        plot.set_yticklabels(
            np.arange(program_args.tile1, program_args.tile2 + 1, step=n_step)
        )

        # Set labels
        # Only do y label for first col
        if plot_col == 0:
            plot.set_ylabel("ant2", size=6)

        # Only do x label for final row
        if plot_row == plot_rows - 1:
            plot.set_xlabel("ant1", size=6)

        plt.setp(
            plot.get_xticklabels(),
            rotation=90,
            ha="right",
            va="center",
            rotation_mode="anchor",
        )

        # Increment so we know which plot we are on
        if plot_col < plot_cols - 1:
            plot_col = plot_col + 1
        else:
            plot_row = plot_row + 1
            plot_col = 0

    print("Saving figure...")
    if program_args.correlator_version == MWAVersion.CorrMWAXv2.value:
        filename = "grid_plot_mwax.png"
    else:
        filename = "grid_plot_mwa.png"
    plt.savefig(filename, bbox_inches="tight", dpi=dpi)
    print(f"saved {filename}")
    plt.show()


def do_grid_plot2(
    title, program_args: ViewFITSArgs, plot_grid_data
):  # noqa: C901
    print("Preparing grid plot2...")

    # Work out layout of plots
    plots = program_args.time_step_count
    plot_rows = math.floor(math.sqrt(plots))
    plot_cols = math.ceil(plots / plot_rows)
    plot_row = 0
    plot_col = 0

    # Determine scaling value
    scaling_value: float = np.max(plot_grid_data)

    # Apply log10 and scaling value
    for time_index in range(0, program_args.time_step_count):
        for t1 in range(0, program_args.tile_count):
            for t2 in range(t1, program_args.tile_count):
                value: float = (
                    plot_grid_data[time_index][t2][t1] / scaling_value
                )
                try:
                    if abs(value) < 0.0000001:
                        plot_grid_data[time_index][t2][t1] = 0
                    else:
                        plot_grid_data[time_index][t2][t1] = math.log10(
                            value * 10000000
                        )
                except Exception as e:
                    print(
                        "Exception trying math.log10("
                        f"{value * 10000000} / {scaling_value})"
                    )
                    raise e

    # Now print some stats
    print(
        f"scaling value was: {scaling_value}; now scaled and "
        f"log10'd-> min = {np.min(plot_grid_data)}; "
        f"max = {np.max(plot_grid_data)}"
    )

    fig, ax = plt.subplots(
        figsize=(30, 30),
        nrows=plot_rows,
        ncols=plot_cols,
        squeeze=False,
        sharex="all",
        sharey="all",
        dpi=dpi,
    )
    fig.suptitle(title)

    n_step = 1

    for time_index in range(0, program_args.time_step_count):
        # for t1 in range(0, program_args.tile_count):
        #    for t2 in range(t1, program_args.tile_count):
        #        print(time_index, program_args.tile1 + t1,
        #        program_args.tile1 + t2, plot_grid_data[time_index][t2][t1])

        print(f"Adding data points for plot({time_index})...")

        # Get the current plot
        plot = ax[plot_row][plot_col]

        plot.imshow(
            plot_grid_data[time_index], cmap="inferno", interpolation="None"
        )

        plot.set_title(f"t={time_index+program_args.time_step1}", size=6)
        plot.set_xticks(np.arange(program_args.tile_count, step=n_step))
        plot.set_yticks(np.arange(program_args.tile_count, step=n_step))
        plot.set_xticklabels(
            np.arange(program_args.tile1, program_args.tile2 + 1, step=n_step)
        )
        plot.set_yticklabels(
            np.arange(program_args.tile1, program_args.tile2 + 1, step=n_step)
        )

        # Set labels
        # Only do y label for first col
        if plot_col == 0:
            plot.set_ylabel("ant2", size=6)

        # Only do x label for final row
        if plot_row == plot_rows - 1:
            plot.set_xlabel("ant1", size=6)

        plt.setp(
            plot.get_xticklabels(),
            rotation=90,
            ha="right",
            va="center",
            rotation_mode="anchor",
        )

        # Increment so we know which plot we are on
        if plot_col < plot_cols - 1:
            plot_col = plot_col + 1
        else:
            plot_row = plot_row + 1
            plot_col = 0

    print("Saving figure...")
    if program_args.correlator_version == MWAVersion.CorrMWAXv2.value:
        filename = f"grid_plot2_{program_args.grid_pol}_mwax.png"
    else:
        filename = f"grid_plot2_{program_args.grid_pol}_mwa.png"

    plt.savefig(filename, bbox_inches="tight", dpi=dpi)
    print(f"saved {filename}")
    plt.show()


def do_phase_plot(
    title, program_args: ViewFITSArgs, plot_phase_data_x, plot_phase_data_y
):  # noqa: C901
    print("Preparing phase plot...")

    # Work out layout of plots
    if program_args.phase_plot_one:
        plots = 1
    else:
        plots = program_args.baseline_count

    print(
        f"Timesteps: {program_args.time_step_count}, "
        f"baselines: {program_args.baseline_count}, "
        f"tiles: {program_args.tile_count}, "
        f"channels: {program_args.channel_count}, "
        f"plots: {plots}"
    )

    plot_cols = program_args.tile_count
    plot_rows = program_args.tile_count

    plot_row = 0
    plot_col = 0
    baseline = 0

    fig, ax = plt.subplots(
        figsize=(11.6, 8.3),
        nrows=plot_rows,
        ncols=plot_cols,
        squeeze=False,
        sharex="all",
        sharey="all",
        dpi=dpi,
    )
    fig.suptitle(title)

    for i in range(0, program_args.tile_count):
        for j in range(i, program_args.tile_count):

            if program_args.phase_plot_one:
                if not (i == 0 and j == (program_args.tile_count - 1)):
                    # skip this plot
                    print(f"{i} vs {j} skip")
                    baseline = baseline + 1
                    continue

            print(f"Adding data points for plot({i},{j})...")
            channel_list = range(
                program_args.channel1, program_args.channel2 + 1
            )

            # Get the current plot
            plot = ax[plot_row][plot_col]

            # Do plots
            for t in range(0, program_args.time_step_count):
                # print(program_args.context.num_timesteps)
                # print(f"Time {t}")
                # print("X")
                # print(plot_phase_data_x[t][baseline])
                # print("Y")
                # print(plot_phase_data_y[t][baseline])

                plot.plot(
                    channel_list,
                    plot_phase_data_x[t][baseline],
                    "o",
                    markersize=1,
                    color="blue",
                )
                plot.plot(
                    channel_list,
                    plot_phase_data_y[t][baseline],
                    "o",
                    markersize=1,
                    color="green",
                )

            # Set labels
            # Only do y label for first col
            if plot_col == 0:
                plot.set_ylabel("phase (deg)", size=6)

            # Only do x label for final row
            if plot_row == plot_rows - 1:
                plot.set_xlabel("fine channel", size=6)

            # Ensure Y axis goes from -180 to 180
            plot.set_ylim([-180, 180])

            tile1_name = program_args.context.metafits_context.antennas[
                i + program_args.tile1
            ].tile_name
            tile2_name = program_args.context.metafits_context.antennas[
                j + program_args.tile1
            ].tile_name
            plot.set_title(f"{tile1_name} v {tile2_name}", size=6, pad=2)

            # Increment so we know which plot we are on
            if plot_col < plot_cols - 1:
                plot_col = plot_col + 1
            else:
                plot_row = plot_row + 1
                plot_col = plot_row

            # Increment baseline
            baseline = baseline + 1

    print("Saving figure...")
    # Save final plot to disk
    if program_args.correlator_version.value == MWAVersion.CorrMWAXv2.value:
        filename = "phase_plot_mwax.png"
    else:
        filename = "phase_plot_mwa.png"

    plt.savefig(filename, bbox_inches="tight", dpi=dpi)
    print(f"saved {filename}")
    plt.show()


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("filename", help="fits filename")
    parser.add_argument(
        "-m", "--metafits", required=True, help="Path to the metafits file."
    )
    parser.add_argument(
        "-t1",
        "--timestep1",
        required=False,
        help="timestep start (1 based index)",
        default=1,
        type=int,
    )
    parser.add_argument(
        "-t2",
        "--timestep2",
        required=False,
        help="timestep end (defaults to last index)",
        default=-1,
        type=int,
    )
    parser.add_argument(
        "-a1",
        "--ant1",
        required=False,
        help="antenna (start)",
        default=-1,
        type=int,
    )
    parser.add_argument(
        "-a2",
        "--ant2",
        required=False,
        help="antenna (end)",
        default=-1,
        type=int,
    )
    parser.add_argument(
        "-c1",
        "--channel1",
        required=False,
        help="fine channel number (start)",
        default=-1,
        type=int,
    )
    parser.add_argument(
        "-c2",
        "--channel2",
        required=False,
        help="fine channel number (end)",
        default=-1,
        type=int,
    )
    parser.add_argument(
        "-a",
        "--autosonly",
        required=False,
        help="Only output the auto correlations",
        action="store_true",
    )
    parser.add_argument(
        "-p",
        "--ppdplot",
        required=False,
        help="Create a ppd plot",
        action="store_true",
    )
    parser.add_argument(
        "-p2",
        "--ppdplot2",
        required=False,
        help="Create a ppd plot that does not sum across all "
        "baselines. ie it plots all baselines",
        action="store_true",
    )
    parser.add_argument(
        "-g",
        "--gridplot",
        required=False,
        help="Create a grid / baseline plot",
        action="store_true",
    )
    parser.add_argument(
        "-g2",
        "--gridplot2",
        required=False,
        help="Create a grid / baseline plot but show a single "
        "pol (XX,XY,YX,YY) for each tile. Use gridpol "
        "to specify",
        action="store_true",
    )
    parser.add_argument(
        "-gp",
        "--gridpol",
        required=False,
        help="If gridplot2 used, use this to specify the pol. "
        "Default is 'XX'",
        default="XX",
    )
    parser.add_argument(
        "-ph",
        "--phaseplot_all",
        required=False,
        help="Will do a phase plot for all baselines for "
        "given antennas and timesteps",
        action="store_true",
    )

    parser.add_argument(
        "-ph1",
        "--phaseplot_one",
        required=False,
        help="Will do a phase plot for given baseline " "and timesteps",
        action="store_true",
    )

    parser.add_argument(
        "-o",
        "--mode",
        required=True,
        help="How to interpret a1 and a2: RANGE or BASELINE",
    )

    parser.add_argument(
        "-dr",
        "--dump-raw",
        required=False,
        help="Dump the raw data",
        action="store_true",
    )
    parser.add_argument(
        "-dp",
        "--dump-plot",
        required=False,
        help="Dump the plot data",
        action="store_true",
    )
    args = vars(parser.parse_args())

    parsed_args = ViewFITSArgs(args)

    peek_fits(parsed_args)
