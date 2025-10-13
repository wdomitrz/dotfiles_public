#!/usr/bin/env sh

xinput list --id-only \
  | xargs --max-args 1 set_touchpad.sh "$@" --touchpad-name 2> /dev/null || true
