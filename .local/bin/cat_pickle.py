#!/usr/bin/env python3

import pickle
import sys
from pathlib import Path


def main():
    for f in map(Path, sys.argv[1:]):
        data = pickle.load(f.open("rb"))  # pyright: ignore[reportAny]
        try:
            print(data)  # pyright: ignore[reportAny]
        except BrokenPipeError:
            pass


if __name__ == "__main__":
    main()
