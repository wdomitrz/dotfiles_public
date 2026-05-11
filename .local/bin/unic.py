#!/usr/bin/env python3
import argparse
import unicodedata
from dataclasses import dataclass
from typing import Self, cast


@dataclass(kw_only=True, frozen=True)
class Args:
    c: int

    @classmethod
    def from_args(cls, argv: list[str] | None = None) -> Self:
        parser = argparse.ArgumentParser()
        _ = parser.add_argument("c", type=int)
        args = parser.parse_args(argv)
        return cls(c=cast(int, args.c))

    def run(self) -> int:
        c = chr(self.c)
        try:
            print(f"{c!r}")
            print(f"'{c}'")
            print(unicodedata.name(c))
        except ValueError:
            print(f"No description for {c!r}")
        return 0


if __name__ == "__main__":
    raise SystemExit(Args.from_args().run())
