#!/usr/bin/env sh
xrandr -q | grep "^screen connected" > /dev/null ||
  xrandr -q | grep -E "^Virtual-[0-9]+ connected" > /dev/null ||
  xrandr -q | grep -E "^rdp[0-9]+ connected" > /dev/null
