#!/usr/bin/env python3
import sys
from pathlib import Path

import pandas as pd


def main():
    for f in map(Path, sys.argv[1:]):
        df = pd.read_parquet(f)
        try:
            df.to_csv(sys.stdout, index=False)
        except BrokenPipeError:
            pass


if __name__ == "__main__":
    main()
