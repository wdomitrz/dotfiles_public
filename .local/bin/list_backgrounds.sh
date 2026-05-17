#!/usr/bin/env sh
exec find -L "${HOME}/.local/share/backgrounds/backgrounds.dir/" -type f \( \
  -iname "*.png" \
  -o -iname "*.jpg" \
  -o -iname "*.heic" \
  \)
