#!/usr/bin/env python3
import json
from dataclasses import dataclass
from urllib.parse import parse_qsl, urlparse

import typer


@dataclass(frozen=True, kw_only=True)
class Args:
    def __post_init__(self) -> None:
        return self.main()

    url: str

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

    def main(self) -> None:

        print(
            json.dumps(
                self.to_info(self.url),
                indent=2,
            )
        )


if __name__ == "__main__":
    typer.run(Args)
