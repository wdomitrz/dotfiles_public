#!/usr/bin/env sh
xclip -out -selection clipboard \
  | exec qr.rs --quiet-zone 1 "$@"
