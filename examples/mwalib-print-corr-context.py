#!/usr/bin/env python

import argparse
import mwalib


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "-m", "--metafits", required=True, help="Path to the metafits file."
    )
    parser.add_argument("gpuboxes", nargs="*", help="Paths to the gpubox files.")
    args = parser.parse_args()

    with mwalib.CorrelatorContext(args.metafits, args.gpuboxes) as context:
        print(context)
