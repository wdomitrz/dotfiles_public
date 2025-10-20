#!/usr/bin/env sh
exec uv run --with pyftpdlib python -m pyftpdlib --write "$@"
