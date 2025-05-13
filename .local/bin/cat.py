#!/usr/bin/env python3
import argparse
import sys
from pathlib import Path
from typing import Any


def cat(fp: Path, *, sep: str) -> Any:
    match fp.suffix:
        case ".parquet" | ".pqt" | ".pq":
            import pandas as pd

            return pd.read_parquet(fp).to_csv(sep=sep, index=False)
        case ".pickle" | ".pkl":
            import pickle

            return pickle.loads(fp.read_bytes())
        case ".csv":
            import pandas as pd

            return pd.read_csv(fp).to_csv(sep=sep, index=False)
        case suffix:
            raise NotImplementedError(f"Unsupported {suffix=} for {fp}")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("paths", type=Path, nargs="+")
    parser.add_argument("--sep", type=str, default="\t")
    args = parser.parse_args()

    for fp in args.paths:
        try:
            data = cat(fp, sep=args.sep)
            print(data)
        except NotImplementedError as e:
            print(e, file=sys.stderr)
        except BrokenPipeError:
            pass


if __name__ == "__main__":
    main()
