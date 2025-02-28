#!/usr/bin/env python3

import pickle
import sys

for f in sys.argv[1:]:
    with open(f, "rb") as file:
        data = pickle.load(file)
    try:
        print(data)
    except BrokenPipeError:
        pass
