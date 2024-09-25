#!/usr/bin/env python
import sys
from sys import implementation, stdout
from mwalib import __version__

print(
    implementation,
    __version__,
    file=stdout,
)
