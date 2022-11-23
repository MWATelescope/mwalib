#!/usr/bin/env python
#
# pymwalib examples/print-context - run through all of pymwalib's objects
#
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.
#
import argparse

from pymwalib.common import MWAVersion
from pymwalib.correlator_context import CorrelatorContext
from pymwalib.metafits_context import MetafitsContext
from pymwalib.version import check_mwalib_version
from pymwalib.voltage_context import VoltageContext

if __name__ == "__main__":
    # ensure we have a compatible mwalib first
    # You can skip this if you want, but your first pymwalib call will raise an error. Best trap it here
    # and provide a nice user message
    try:
        check_mwalib_version()
    except Exception as e:
        print(e)
        exit(1)

    parser = argparse.ArgumentParser()

    parser.add_argument(
        "-m", "--metafits", required=True, help="Path to the metafits file."
    )
    parser.add_argument(
        "datafiles",
        nargs="*",
        help="Paths to the gpubox or voltage data files (or neither).",
    )
    args = parser.parse_args()

    context = None

    # Check what we have for the data files
    if len(args.datafiles) == 0:
        # We invoke a metafits context
        print(
            "Only metafits file provided, assuming Legacy Correlator interpretation of metafits."
        )
        context = MetafitsContext(args.metafits, MWAVersion.CorrLegacy)
    else:
        corr_suffixes = [x for x in args.datafiles if x[-5:] == ".fits"]
        dat_suffixes = [x for x in args.datafiles if x[-4:] == ".dat"]
        sub_suffixes = [x for x in args.datafiles if x[-4:] == ".sub"]

        if len(corr_suffixes) + len(dat_suffixes) + len(sub_suffixes) == 0:
            print("Error- no .fits, .dat or .sub files provided")
            exit(-2)
        elif len(corr_suffixes) > 0 and len(dat_suffixes) + len(sub_suffixes) == 0:
            print(f"{len(corr_suffixes)} correlator/gpubox files detected")
            context = CorrelatorContext(args.metafits, args.datafiles)
        elif (
            len(dat_suffixes) > 0 and len(corr_suffixes) + len(sub_suffixes) == 0
        ) or (len(sub_suffixes) > 0 and len(corr_suffixes) + len(dat_suffixes) == 0):
            print(f"{len(dat_suffixes)} voltage data files detected")
            context = VoltageContext(args.metafits, args.datafiles)
        else:
            print("Error: Combination of different data files supplied.")
            exit(-3)

    # Test the debug "display" method
    print("\nTesting Display method:")
    context.display()
