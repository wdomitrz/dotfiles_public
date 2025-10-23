#!/usr/bin/env -S uv run --script
import sys

nato_alphabet: dict[str, str] = {
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


# Function to convert input string to phonetic alphabet
def nato_convert(text: str) -> list[str]:
    return [nato_alphabet.get(c, c) for c in text.lower()]


# Read from standard input or command line arguments
if len(sys.argv) > 1:
    print(*nato_convert(" ".join(sys.argv[1:])), sep="\t")
else:
    for line in sys.stdin:
        print(*nato_convert(line), sep="\t")
