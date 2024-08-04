#!/usr/bin/env python3

import sys

import pandas as pd

for f in sys.argv[1:]:
    df = pd.read_parquet(f)
    try:
        df.to_csv(sys.stdout, index=False)
    except BrokenPipeError:
        pass
