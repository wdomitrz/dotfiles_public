#!/usr/bin/env -S uv run --script
#
# /// script
# dependencies = [
#     "typer",
# ]
# ///
#

import json
from dataclasses import dataclass
from urllib.parse import parse_qsl, urlparse

import typer  # pyright: ignore[reportMissingImports]


@dataclass(frozen=True, kw_only=True)
class Args:
    url: str

    def __post_init__(self) -> None:
        return main(self)


def main(args: Args) -> None:
    parsed = urlparse(args.url)

    print(
        json.dumps(
            {
                k: v
                for k, v in dict(
                    original=args.url,
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
            },
            indent=2,
        )
    )


if __name__ == "__main__":
    typer.run(Args)  # pyright: ignore[reportUnknownMemberType]
