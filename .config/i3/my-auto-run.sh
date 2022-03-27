#!/usr/bin/env sh
(check-if-remote || dex --autostart --environment i3) &
(check-if-remote || nm-applet) &
(check-if-remote || redshift-gtk) &
compton &
fusuma &
geoclue-2-agent &
gtk-launch org.kde.kdeconnect.nonplasma &
blueman-applet &
/usr/lib/x86_64-linux-gnu/xfce4/notifyd/xfce4-notifyd &
lxpolkit &
xfce4-power-manager &
i3-screen-locker &
ibus-daemon --daemonize --replace &
i3-xrl &
set-keymap &
