#!/usr/bin/env sh
set -eu

DEFAULT_OPTIONS_FILE="${HOME}"/.config/dpi/default_dpi_options.sorted_numeric.txt
DPI="$(
  (
    set_display.py show_dpi
    [ -f "${DEFAULT_OPTIONS_FILE}" ] && cat "${DEFAULT_OPTIONS_FILE}"
  ) \
    | awk '!x[$0]++' \
    |
    # Deduplicate lines
    rofi -dmenu -p dpi
)" set_display.py set_dpi
