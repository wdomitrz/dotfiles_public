#!/usr/bin/env python3
#
# /// script
# dependencies = [
#   "typing_extensions",
# ]
# ///

from __future__ import annotations

import argparse
import json
from dataclasses import dataclass
from typing import TypeAlias, cast
from urllib.parse import parse_qsl, urlparse

from typing_extensions import Self

QueryValue: TypeAlias = str | list[str]
QueryInfo: TypeAlias = dict[str, QueryValue]
RawQueryInfo: TypeAlias = dict[str, list[str]]
URLInfo: TypeAlias = dict[str, int | str | QueryInfo]


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
    def to_info(cls, url: str) -> URLInfo:
        parsed = urlparse(url)
        raw_query: RawQueryInfo = {}
        for key, value in parse_qsl(parsed.query, keep_blank_values=True):
            raw_query.setdefault(key, []).append(value)
        query: QueryInfo = {
            key: values[0] if len(values) == 1 else values
            for key, values in raw_query.items()
        }
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
                query=query,
                hash=parsed.fragment,
            ).items()
            if v is not None and (not isinstance(v, (str, dict)) or len(v) != 0)
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
