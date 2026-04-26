#!/usr/bin/env python3
import sys
import unicodedata

assert len(sys.argv) == 2

c = chr(int(sys.argv[1], 0))

print(c)
try:
    print(unicodedata.name(c))
except ValueError:
    print(f"No description for {c}")
