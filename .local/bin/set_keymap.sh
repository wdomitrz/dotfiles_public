#!/usr/bin/env sh

pkill -9 --full xcape || true

# caps to ctrl
setxkbmap -option
setxkbmap -option ctrl:nocaps

# ctrl as esc when tapped
xcape -e 'Control_L=Escape'

# space as win
xmodmap -e "clear lock"
xmodmap -e "keycode 65 = Super_L"

# win as space
xmodmap -e "keycode any = space"
xcape -e 'Super_L=space'
