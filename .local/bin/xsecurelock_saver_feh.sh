#!/usr/bin/env bash
set -eou pipefail

"${HOME}"/.local/bin/list_backgrounds.sh \
  | sort --random-sort | head -n 1 \
  | exec xargs feh \
    --window-id "${XSCREENSAVER_WINDOW:?XSCREENSAVER_WINDOW not set}" \
    --fullscreen --zoom fill --no-xinema
