#!/usr/bin/bash

# --output-document uses the last passed value
wget_default_options=(
  "--no-verbose" "--show-progress" "--output-document=-" "--max-redirect=0")

exec wget "${wget_default_options[@]}" "$@"
