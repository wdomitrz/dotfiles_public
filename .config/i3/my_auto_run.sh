#!/usr/bin/env sh
(check_if_remote || dex --autostart --environment i3) &
fusuma &
geoclue-2-agent &
i3_screen_locker &
i3status_run_file_updaters.sh &
i3_theme_detection &
set_keymap &
xfce4-power-manager &
i3_xrl --no-set-background &&
    (
        (check_if_remote || (
            blueman-applet &
            env LANGUAGE=pl_PL.utf-8 signal-desktop --use-tray-icon --start-in-tray &
            nm-applet &
            redshift-gtk &
        )) &
        ([ -f "${HOME}/.config/i3/my_auto_run.local.sh" ] && "${HOME}/.config/i3/my_auto_run.local.sh") &
        gtk-launch org.kde.kdeconnect.nonplasma &
        ibus-daemon --daemonize --replace &
        /usr/lib/policykit-1-gnome/polkit-gnome-authentication-agent-1 &
        (
            systemctl --user stop dunst
            (
                /usr/lib/x86_64-linux-gnu/xfce4/notifyd/xfce4-notifyd &
                /usr/lib64/xfce4/notifyd/xfce4-notifyd &
            )
        ) &
        while :; do
            set_background
            sleep 60
        done &
    )
