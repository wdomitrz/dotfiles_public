#!/usr/bin/env -S uv run --script
#
# /// script
# dependencies = [
#     "pandas",
#     "typer",
# ]
# ///
#
# pyright: reportMissingImports = false
# pyright: reportUnknownMemberType = false
# pyright: reportUnknownVariableType = false

import sys
from dataclasses import dataclass
from pathlib import Path
from typing import cast

import typer


def cat(fp: Path, *, sep: str, weights_only: bool, pd_show_index: bool) -> object:
    match fp.suffix:
        case ".parquet" | ".pqt" | ".pq":
            import pandas as pd

            return pd.read_parquet(fp).to_csv(sep=sep, index=pd_show_index)
        case ".pickle" | ".pkl":
            import pickle

            return cast(object, pickle.loads(fp.read_bytes()))
        case ".csv":
            import pandas as pd

            return pd.read_csv(fp).to_csv(sep=sep, index=pd_show_index)
        case ".pt":
            import torch

            return torch.load(fp, map_location="cpu", weights_only=weights_only)
        case suffix:
            raise NotImplementedError(f"Unsupported {suffix=} for {fp}")


@dataclass(frozen=True, kw_only=True)
class Args:
    paths: list[Path]
    sep: str = "\t"
    weights_only: bool = False
    pd_show_index: bool = False

    def __post_init__(self) -> None:
        return main(self)


def main(args: Args) -> None:
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
    typer.run(Args)
