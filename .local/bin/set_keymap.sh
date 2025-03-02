#!/usr/bin/env sh

# Clear all options
pkill -9 --full xcape || true
xmodmap -e "clear lock"

# caps -> esc ctrl
setxkbmap -option ctrl:nocaps
xcape -e 'Control_L=Escape'

# space win
xmodmap -e "keycode 65 = Super_L"
xmodmap -e "keycode any = space" && xcape -e 'Super_L=space'

# tab alt
xmodmap -e "keycode 23 = Alt_L"
xmodmap -e "keycode any = Tab" && xcape -e 'Alt_L=Tab'
