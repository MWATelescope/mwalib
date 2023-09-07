#!/usr/bin/env python
#
# pymwalib examples/print-tile-info - print tile info given a metafits file.
#
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.
#
#
import argparse
import mwalib

if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "-m", "--metafits", required=True, help="Path to the metafits file."
    )
    args = parser.parse_args()

    metafits_filename = args.metafits

    with mwalib.MetafitsContext(metafits_filename) as context:
        print("ant\tpol\tname\trec\tslot\telec len (m)\tflagged?")
        for r in context.rf_inputs:
            print(
                f"{r.ant}\t{r.pol}\t{r.tile_name}\t{r.rec_number}\t{r.rec_slot_number}\t{r.electrical_length_m}\t{1 if r.flagged else 0}"
            )
