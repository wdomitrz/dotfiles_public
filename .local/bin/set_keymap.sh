#!/usr/bin/env sh

# Clear all options
pkill --signal SIGKILL --full xcape || true # SIGKILL is the singla number 9

# caps -> ctrl
setxkbmap -option ctrl:nocaps
# ctrl -> esc (when tapped)
xcape -e 'Control_L=Escape'
