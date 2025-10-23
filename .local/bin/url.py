#!/usr/bin/env -S uv run --script
import argparse
import json
from urllib.parse import parse_qsl, urlparse

parser = argparse.ArgumentParser(description="Print out the parts of a URL")
parser.add_argument("url", type=str, help="The URL to parse")
args = parser.parse_args()

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
            if v is not None and len(v) != 0
        },
        indent=2,
    )
)
