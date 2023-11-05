#!/usr/bin/env bash

function set_shell() {
    shell="$1"

    sudo chsh -s "${shell}" "${USER}" ||
        chsh -s "${shell}"
}

function set_shell_bash() {
    set_shell /usr/bin/bash
}

function set_shell_zsh() {
    set_shell /usr/bin/zsh
}

function disable_core_file() {
    ulimit -c 0
}

function update_tldr() {
    tldr --update
}

function add_bookmarks() {
    [[ -f "${HOME}/.config/gtk-3.0/bookmarks" ]] ||
        (mkdir --parents "${HOME}"/.config/gtk-3.0 &&
            touch "${HOME}"/.config/gtk-3.0/bookmarks)

    for d in Documents Downloads Music Pictures Videos; do
        grep --quiet "${d}" "${HOME}"/.config/gtk-3.0/bookmarks ||
            echo "file://$(pwd)/${d}" >>"${HOME}"/.config/gtk-3.0/bookmarks
    done
}

function configure_nautilus_open_terminal() {
    # Open kitty from nautilus
    glib-compile-schemas "${HOME}"/.local/share/glib-2.0/schemas/
    [[ -z ${TERMINAL+variable_unset} ]] && echo "Variable TERMINAL is not set" && return 1
    DISPLAY=:0 gsettings set com.github.stunkymonkey.nautilus-open-any-terminal terminal "${TERMINAL}"
}

function configure_transmission() {
    [[ -f "${HOME}/.config/transmission/settings.bck.json" ]] &&
        cp "${HOME}"/.config/transmission/settings.bck.json "${HOME}"/.config/transmission/settings.json
}

function install_code_extensions() {
    "${HOME}"/.local/bin/install_code_extensions
}

function main() {
    set -euo pipefail
    set -x

    set_shell_bash
    update_tldr
    configure_nautilus_open_terminal
    configure_transmission
    install_code_extensions
    "${HOME}"/.local/bin/set_theme dark
}

if [[ "$#" -ne 1 ]] || [[ "${1}" != "--source-only" ]]; then
    main "${@}"
fi
