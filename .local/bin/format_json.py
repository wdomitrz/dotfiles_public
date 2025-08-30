#!/usr/bin/env python3
################################################################
# Copyright (c) 2025 Witalis Domitrz <witekdomitrz@gmail.com>
# AGPL License
################################################################
import argparse
import json
import sys
from dataclasses import dataclass
from typing import Any, NamedTuple


@dataclass(frozen=True, kw_only=True)
class JSONDumper:
    indent: str
    line_length: int

    class PartialDump(NamedTuple):
        single_lined: str
        expanded: str

    def _dumps_list(self, xs: list, *, indent: str) -> PartialDump:
        if len(xs) == 0:
            return JSONDumper.PartialDump(single_lined="[]", expanded="[]")

        child_indent = indent + self.indent

        elems = [self._dumps_helper(x, indent=child_indent) for x in xs]

        def get_indented_child(x: JSONDumper.PartialDump, *, last: bool) -> str:
            if len(x.single_lined) <= self.line_length - (
                len(child_indent) + (0 if last else 1)
            ):
                x_chosen = x.single_lined
            else:
                x_chosen = x.expanded

            return f"{child_indent}{x_chosen}"

        return JSONDumper.PartialDump(
            single_lined="".join(["[", ", ".join(x.single_lined for x in elems), "]"]),
            expanded="\n".join(
                [
                    "[",
                    ",\n".join(
                        get_indented_child(x, last=(i == len(elems) - 1))
                        for i, x in enumerate(elems)
                    ),
                    indent + "]",
                ]
            ),
        )

    def _dumps_dict(self, xs: dict, *, indent: str) -> PartialDump:
        if len(xs) == 0:
            return JSONDumper.PartialDump(single_lined="{}", expanded="{}")

        child_indent = indent + self.indent

        elems = {
            json.dumps(k, indent=None): self._dumps_helper(v, indent=child_indent)
            for k, v in xs.items()
        }

        def get_indented_child(k: str, v: JSONDumper.PartialDump, *, last: bool) -> str:
            if len(v.single_lined) <= self.line_length - (
                len(child_indent) + len(k) + (2 if last else 3)
            ):
                v_chosen = v.single_lined
            else:
                v_chosen = v.expanded

            return f"{child_indent}{k}: {v_chosen}"

        return JSONDumper.PartialDump(
            single_lined="".join(
                [
                    "{ ",
                    ", ".join(f"{k}: {v.single_lined}" for k, v in elems.items()),
                    " }",
                ]
            ),
            expanded="\n".join(
                [
                    "{",
                    ",\n".join(
                        get_indented_child(k, v, last=(i == len(elems) - 1))
                        for i, (k, v) in enumerate(elems.items())
                    ),
                    indent + "}",
                ]
            ),
        )

    def _dumps_helper(self, x: Any, *, indent: str) -> PartialDump:
        if isinstance(x, list):
            return self._dumps_list(x, indent=indent)
        elif isinstance(x, dict):
            return self._dumps_dict(x, indent=indent)
        else:
            r = json.dumps(x)
            return JSONDumper.PartialDump(single_lined=r, expanded=r)

    def dumps(self, x: Any) -> str:
        r = self._dumps_helper(x, indent="")
        if len(r.single_lined) <= self.line_length:
            return r.single_lined
        else:
            return r.expanded


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--indent", type=int, default=2)
    parser.add_argument("--sort_keys".replace("_", "-"), type=bool, default=False)
    parser.add_argument("--line_length".replace("_", "-"), type=int, default=80)
    return parser.parse_args()


def main():
    args = parse_args()
    data = json.loads(sys.stdin.read())
    if args.sort_keys:
        data = json.loads(json.dumps(data, sort_keys=True, indent=None))
    print(
        JSONDumper(indent=" " * args.indent, line_length=args.line_length).dumps(data)
    )


if __name__ == "__main__":
    main()
