#!/usr/bin/env python3
#
# pyright: reportMissingImports = false
# pyright: reportMissingTypeStubs = false
# pyright: reportUnknownMemberType = false
# pyright: reportUnknownVariableType = false

import argparse
import pickle
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import cast

import pandas as pd
from typing_extensions import Self


def cat(fp: Path, *, sep: str, weights_only: bool, pd_show_index: bool) -> object:
    match fp.suffix:
        case ".parquet" | ".pqt" | ".pq":
            return pd.read_parquet(fp).to_csv(sep=sep, index=pd_show_index)
        case ".pickle" | ".pkl":
            return cast(object, pickle.loads(fp.read_bytes()))
        case ".csv":
            return pd.read_csv(fp).to_csv(sep=sep, index=pd_show_index)
        case ".pt":
            import torch  # noqa: PLC0415

            return cast(
                object, torch.load(fp, map_location="cpu", weights_only=weights_only)
            )
        case suffix:
            raise NotImplementedError(f"Unsupported {suffix=} for {fp}")


@dataclass(frozen=True, kw_only=True)
class Args:
    paths: list[Path]
    sep: str
    weights_only: bool
    pd_show_index: bool

    @classmethod
    def from_args(cls, argv: list[str] | None = None) -> Self:
        parser = argparse.ArgumentParser()
        _ = parser.add_argument("paths", nargs="+", type=Path)
        _ = parser.add_argument("--sep", default="\t")
        _ = parser.add_argument(
            "--weights-only",
            action=argparse.BooleanOptionalAction,
            default=False,
        )
        _ = parser.add_argument(
            "--pd-show-index",
            action=argparse.BooleanOptionalAction,
            default=False,
        )
        args = parser.parse_args(argv)
        return cls(
            paths=cast(list[Path], args.paths),
            sep=cast(str, args.sep),
            weights_only=cast(bool, args.weights_only),
            pd_show_index=cast(bool, args.pd_show_index),
        )

    def run(self) -> int:
        for fp in self.paths:
            try:
                data = cat(
                    fp,
                    sep=self.sep,
                    weights_only=self.weights_only,
                    pd_show_index=self.pd_show_index,
                )
                print(data)
            except NotImplementedError as e:  # noqa: PERF203
                print(e, file=sys.stderr)
            except BrokenPipeError:
                pass
        return 0


if __name__ == "__main__":
    raise SystemExit(Args.from_args().run())
