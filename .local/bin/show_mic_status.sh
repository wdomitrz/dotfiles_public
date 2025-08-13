#!/usr/bin/env sh
pactl get-source-mute @DEFAULT_SOURCE@ |
  sed -e 's/Mute: no/🎤/' -e 's/Mute: yes/🚫🎤/' |
  xargs --delimiter='\n' notify-send
