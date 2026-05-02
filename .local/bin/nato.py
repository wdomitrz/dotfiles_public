#!/usr/bin/env python3
#
# /// script
# dependencies = [
#   "typer",
# ]
# ///

import sys
from dataclasses import dataclass
from typing import ClassVar

import typer


@dataclass(frozen=True, kw_only=True)
class Nato:
    def __post_init__(self) -> None:
        return self.main()

    words: list[str]

    nato_alphabet: ClassVar[dict[str, str]] = {
        "0": "Zero",
        "1": "One",
        "2": "Two",
        "3": "Three",
        "4": "Four",
        "5": "Five",
        "6": "Six",
        "7": "Seven",
        "8": "Eight",
        "9": "Nine",
        "a": "Alfa",
        "b": "Bravo",
        "c": "Charlie",
        "d": "Delta",
        "e": "Echo",
        "f": "Foxtrot",
        "g": "Golf",
        "h": "Hotel",
        "i": "India",
        "j": "Juliett",
        "k": "Kilo",
        "l": "Lima",
        "m": "Mike",
        "n": "November",
        "o": "Oscar",
        "p": "Papa",
        "q": "Quebec",
        "r": "Romeo",
        "s": "Sierra",
        "t": "Tango",
        "u": "Uniform",
        "v": "Victor",
        "w": "Whiskey",
        "x": "X-ray",
        "y": "Yankee",
        "z": "Zulu",
    }

    @classmethod
    def nato_convert(cls, text: str) -> list[str]:
        return [cls.nato_alphabet.get(c, c) for c in text.lower()]

    def main(self):
        match self.words:
            case ["-"]:
                for line in sys.stdin:
                    print(*self.nato_convert(line), sep="\t")
            case words:
                print(*self.nato_convert(" ".join(words)), sep="\t")


Args = Nato

if __name__ == "__main__":
    typer.run(Args)
