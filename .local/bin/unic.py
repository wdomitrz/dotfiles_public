#!/usr/bin/env python3
#
# /// script
# dependencies = [
#   "typer",
# ]
# ///

import unicodedata
from dataclasses import dataclass

import typer


@dataclass(kw_only=True, frozen=True)
class Args:
    def __post_init__(self) -> None:
        return self.main()

    c: int

    def main(self) -> None:
        c = chr(self.c)
        try:
            print(f"{c!r}")
            print(f"'{c}'")
            print(unicodedata.name(c))
        except ValueError:
            print(f"No description for {c!r}")


if __name__ == "__main__":
    typer.run(Args)
