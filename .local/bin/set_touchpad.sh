#!/usr/bin/env sh
set -eu

# Default values
reversed=false

while [ $# -gt 0 ]; do
  case "$1" in
    --touchpad-name) touchpad_name="$2" && shift 2 ;;
    --reversed) reversed="$2" && shift 2 ;;
    *) echo "Unknown param $1" && exit 1 ;;
  esac
done

xinput set-prop "${touchpad_name}" 'libinput Tapping Enabled' 1
if ! "${reversed}"; then
  xinput set-prop "${touchpad_name}" 'Coordinate Transformation Matrix' 1 0 0 0 1 0 0 0 1
  xinput set-prop "${touchpad_name}" 'libinput Natural Scrolling Enabled' 0
else
  xinput set-prop "${touchpad_name}" 'Coordinate Transformation Matrix' -1 0 1 0 -1 1 0 0 1
  xinput set-prop "${touchpad_name}" 'libinput Natural Scrolling Enabled' 1
fi
