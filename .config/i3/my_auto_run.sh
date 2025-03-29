#!/usr/bin/env sh
(check_if_remote.sh || dex --autostart --environment i3) &
fusuma &
geoclue-2-agent &
i3_screen_locker.sh &
i3status_run_file_updaters.sh &
i3_theme_detection &
( (cat "${HOME}"/.config/set_audio/"$(hostname)".txt || true) | xargs --no-run-if-empty set_audio.sh) &
set_keymap.sh &
xfce4-power-manager &
i3_xrl --no-set-background &&
    (
        (check_if_remote.sh || (
            blueman-applet &
            signal-desktop &
            nm-applet &
            redshift-gtk &
        )) &
        ([ -f "${HOME}/.config/i3/my_auto_run.local.sh" ] && "${HOME}/.config/i3/my_auto_run.local.sh") &
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
