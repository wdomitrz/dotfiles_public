#!/usr/bin/env sh
xclip -out -selection clipboard |
  exec xargs --no-run-if-empty --null qrterminal -q 0 "$@"
