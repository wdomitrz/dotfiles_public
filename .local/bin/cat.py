#!/usr/bin/env python3
#
# /// script
# dependencies = [
#   "pandas",
#   "torch",
#   "typing_extensions",
# ]
# ///
#
# pyright: reportMissingImports = false
# pyright: reportMissingTypeStubs = false
# pyright: reportUnknownMemberType = false
# pyright: reportUnknownVariableType = false

import argparse
from dataclasses import dataclass
from pathlib import Path
from typing import cast

from typing_extensions import Self


@dataclass(frozen=True, kw_only=True)
class Args:
    path: Path
    sep: str
    weights_only: bool
    pd_show_index: bool

    def cat(self) -> object:
        match self.path.suffix:
            case ".parquet" | ".pqt" | ".pq":
                import pandas as pd  # noqa: PLC0415

                return pd.read_parquet(self.path).to_csv(
                    sep=self.sep, index=self.pd_show_index
                )
            case ".pickle" | ".pkl":
                import pickle  # noqa: PLC0415

                return cast(object, pickle.loads(self.path.read_bytes()))
            case ".csv":
                import pandas as pd  # noqa: PLC0415

                return pd.read_csv(self.path).to_csv(
                    sep=self.sep, index=self.pd_show_index
                )
            case ".pt":
                import torch  # noqa: PLC0415

                return cast(
                    object,
                    torch.load(
                        self.path, map_location="cpu", weights_only=self.weights_only
                    ),
                )
            case suffix:
                raise NotImplementedError(f"Unsupported {suffix=} for {self.path}")

    @classmethod
    def from_args(cls, argv: list[str] | None = None) -> Self:
        parser = argparse.ArgumentParser()
        _ = parser.add_argument("path", type=Path)
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
            path=cast(Path, args.path),
            sep=cast(str, args.sep),
            weights_only=cast(bool, args.weights_only),
            pd_show_index=cast(bool, args.pd_show_index),
        )

    def run(self) -> int:
        try:
            data = self.cat()
            print(data)
        except BrokenPipeError:
            pass
        return 0


if __name__ == "__main__":
    raise SystemExit(Args.from_args().run())
