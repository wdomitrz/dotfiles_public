#!/usr/bin/env bash

function copy_user_configs() {
    check_integrity_of_tracked_dir "${HOME}"/.config/user_configs &&
        cp --backup=numbered --verbose --recursive "${HOME}"/.config/user_configs/. "${HOME}"/
}

function set_shell() {
    shell="$1"

    sudo chsh -s "${shell}" "${USER}" ||
        chsh -s "${shell}"
}

function set_shell_bash() {
    set_shell /usr/bin/bash
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
            echo "file://$(pwd)/${d}" >> "${HOME}"/.config/gtk-3.0/bookmarks
    done
}

function compile_glib_schemas() {
    glib-compile-schemas "${HOME}"/.local/share/glib-2.0/schemas/
}

function configure_nautilus_open_terminal() {
    compile_glib_schemas
    # Open kitty from nautilus
    [[ -z ${TERMINAL+variable_unset} ]] && echo "Variable TERMINAL is not set" && return 1
    DISPLAY=:0 gsettings set com.github.stunkymonkey.nautilus-open-any-terminal terminal "${TERMINAL}"
}

function install_code_extensions() {
    "${HOME}"/.local/bin/install_code_extensions
}

function config_user_main() {
    set -euo pipefail
    set -x

    copy_user_configs
    set_shell_bash
    compile_glib_schemas
    update_tldr
    configure_nautilus_open_terminal
    install_code_extensions
    "${HOME}"/.local/bin/set_theme light
}

if [[ "$#" -ne 1 ]] || [[ "${1}" != "--source-only" ]]; then
    config_user_main "${@}"
fi
