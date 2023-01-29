#!/usr/bin/env sh
(check-if-remote || dex --autostart --environment i3) &
compton &
fusuma &
geoclue-2-agent &
i3-screen-locker &
set-keymap &
xfce4-power-manager &
i3-xrl &&
    (
        (check-if-remote || (
            blueman-applet &
            nm-applet &
            redshift-gtk &
            env LANGUAGE=pl_PL.utf-8 signal-desktop --use-tray-icon --start-in-tray &
        )) &
        gtk-launch org.kde.kdeconnect.nonplasma &
        ibus-daemon --daemonize --replace &
        mate-polkit &
        /usr/lib/x86_64-linux-gnu/xfce4/notifyd/xfce4-notifyd &
    )
