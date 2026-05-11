#!/usr/bin/env python3

from __future__ import annotations

import argparse
import json
from dataclasses import dataclass
from typing import Self, cast
from urllib.parse import parse_qsl, urlparse


@dataclass(frozen=True, kw_only=True)
class Args:
    url: str

    @classmethod
    def from_args(cls, argv: list[str] | None = None) -> Self:
        parser = argparse.ArgumentParser()
        _ = parser.add_argument("url")
        args = parser.parse_args(argv)
        return cls(url=cast(str, args.url))

    @classmethod
    def to_info(cls, url: str) -> dict[str, int | str | dict[str, str]]:
        parsed = urlparse(url)
        return {
            k: v
            for k, v in dict(
                original=url,
                protocol=parsed.scheme,
                username=parsed.username,
                password=parsed.password,
                hostname=parsed.hostname,
                port=parsed.port,
                path=parsed.path,
                query=dict(parse_qsl(parsed.query)),
                hash=parsed.fragment,
            ).items()
            if v is not None
            and (not isinstance(v, (str, dict, list, tuple)) or len(v) != 0)
        }

    def run(self) -> int:
        print(
            json.dumps(
                self.to_info(self.url),
                indent=2,
            )
        )
        return 0


if __name__ == "__main__":
    raise SystemExit(Args.from_args().run())
