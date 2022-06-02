#!/usr/bin/env bash

function add_user_to_groups {
    sudo usermod -aG docker,input,kvm,lpadmin,audio,netdev,video,libvirt "$USER"
}

function set_shell {
    shell="$1"

    sudo chsh -s "$shell" "$USER" ||
        chsh -s "$shell"
}

function set_shell_bash {
    set_shell /usr/bin/bash
}

function set_shell_zsh {
    set_shell /usr/bin/zsh
}

function set_shell_fish {
    set_shell /usr/bin/fish
}

function disable_core_file {
    ulimit -c 0
}

function disable_core_file {
    tldr --update
}

function create_default_directories {
    default_dirs_location="$HOME/.config/default-dirs/default-dirs.zip"
    if [ -f "$default_dirs_location" ]; then
        cd "$HOME"
        unzip "$default_dirs_location"
        [ -f "$HOME/.config/gtk-3.0/bookmarks" ] || touch "$HOME"/.config/gtk-3.0/bookmarks
        for d in $(zipinfo -1 "$default_dirs_location"); do
            grep --quiet "$d" "$HOME"/.config/gtk-3.0/bookmarks || echo "file://$(pwd)/$d" \
                >>"$HOME"/.config/gtk-3.0/bookmarks
        done
        cd -
    fi
}

function configure_nautilus {
    # Open kitty from nautilus
    glib-compile-schemas "$HOME"/.local/share/glib-2.0/schemas/
    DISPLAY=:0 gsettings set com.github.stunkymonkey.nautilus-open-any-terminal terminal kitty
}

function configure_transmission {
    [ -f "$HOME/.config/transmission/settings.bck.json" ] && cp "$HOME"/.config/transmission/settings.bck.json "$HOME"/.config/transmission/settings.json
}

function set_bacground_switcher {
    if !(crontab -l 2>/dev/null | grep --quiet set-background); then
        (
            crontab -l 2>/dev/null
            echo -n "
# Change background 1 minute
*/1 * * * * DISPLAY=:0 .local/bin/set-background
"
        ) | crontab -
    fi
}

function main {
    set -xue

    add_user_to_groups
    set_shell_zsh
    disable_core_file
    create_default_directories
    configure_nautilus
    configure_transmission
    set_bacground_switcher
}

if [ "$#" -ne 1 ] || [ "${1}" != "--source-only" ]; then
    main "${@}"
fi
