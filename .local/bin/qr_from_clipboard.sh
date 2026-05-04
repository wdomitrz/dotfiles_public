#!/usr/bin/env sh
xclip -out -selection clipboard \
  | exec qr.py --quiet-zone 1 "$@"
