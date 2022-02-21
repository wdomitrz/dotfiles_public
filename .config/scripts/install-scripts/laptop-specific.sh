# Enable charge threshold for tlp
sudo sed -i 's/#\([A-Z]*_CHARGE_THRESH_BAT0=[0-9]*\)$/\1/' /etc/tlp.conf

# Disable sleep with external power
sudo sed -i -E "s/^#?(HandleLidSwitchExternalPower)\s*=\s*[a-z]*/\1=lock/g" /etc/systemd/logind.conf
sudo sed -i -E "s/^#?(HandleLidSwitchDocked)\s*=\s*[a-z]*/\1=lock/g" /etc/systemd/logind.conf

# Tap to click
sudo sed 's/^\(\( *\)Identifier "[a-zA-Z0-9 ]*touchpad[a-zA-Z0-9 ]*" *\)$/\1\n\2Option "Tapping" "on"/' -i /usr/share/X11/xorg.conf.d/40-libinput.conf

# Fix touchpad after sleep
echo '#!/bin/sh

case $1 in
  post)
    /sbin/modprobe -r psmouse && /sbin/modprobe psmouse
  ;;
esac' | sudo tee /lib/systemd/system-sleep/touchpad
