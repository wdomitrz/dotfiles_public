#!/usr/bin/env -S uv run --script
#
# /// script
# dependencies = [
#     "pandas",
# ]
# ///

import argparse
import sys
from pathlib import Path
from typing import Any


def cat(fp: Path, *, sep: str, weights_only: bool, pd_show_index: bool) -> Any:
    match fp.suffix:
        case ".parquet" | ".pqt" | ".pq":
            import pandas as pd

            return pd.read_parquet(fp).to_csv(sep=sep, index=pd_show_index)
        case ".pickle" | ".pkl":
            import pickle

            return pickle.loads(fp.read_bytes())
        case ".csv":
            import pandas as pd

            return pd.read_csv(fp).to_csv(sep=sep, index=pd_show_index)
        case ".pt":
            import torch  # pyright: ignore[reportMissingImports]

            return torch.load(fp, map_location="cpu", weights_only=weights_only)
        case suffix:
            raise NotImplementedError(f"Unsupported {suffix=} for {fp}")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("paths", type=Path, nargs="+")
    parser.add_argument("--sep", type=str, default="\t")
    parser.add_argument("--weights_only".replace("_", "-"), type=bool, default=False)
    parser.add_argument("--pd_show_index".replace("_", "-"), type=bool, default=False)
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    for fp in args.paths:
        try:
            data = cat(
                fp,
                sep=args.sep,
                weights_only=args.weights_only,
                pd_show_index=args.pd_show_index,
            )
            print(data)
        except NotImplementedError as e:
            print(e, file=sys.stderr)
        except BrokenPipeError:
            pass


if __name__ == "__main__":
    main()
