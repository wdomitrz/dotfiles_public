#!/usr/bin/env bash
set +e
set -uo pipefail

function help_and_exit() {
    echo "usage:$0 [dark/light/toggle]"
    exit 1
}

function set_gtk4_theme() {
    dark_or_light="$1"
    if [[ ${dark_or_light} == "dark" ]]; then
        theme_name="Adwaita-dark"
    else
        theme_name="Adwaita"
    fi

    if ! command -v gsettings > /dev/null; then
        return
    fi

    gsettings set org.gnome.desktop.interface gtk-theme "${theme_name}"
    gsettings set org.gnome.desktop.interface color-scheme prefer-"${dark_or_light}"
}

function set_theme_though_links() {
    local -r theme_path="$1"
    local -r dark_or_light="$2"

    local -r theme_dir="$(dirname "${theme_path}")"
    local -r theme_filename="$(basename "${theme_path}")"
    local -r extension="${theme_filename##*.}"
    local -r base_filename="${theme_filename%.*}"

    local -r styled_theme_path="${theme_dir}"/"${base_filename}"_"${dark_or_light}"."${extension}"

    mkdir --parents "${theme_dir}"
    rm --force "${theme_path}"
    ln --symbolic --relative --force "${styled_theme_path}" "${theme_path}"
}

function set_kitty_theme() {
    local -r theme_path="${HOME}"/.config/kitty/theme.conf
    set_theme_though_links "${theme_path}" "$@"

    (pidof kitty || true) |
        xargs --no-run-if-empty ps --format pid:1,tty:1 --no-headers --pid |
        grep "?" | cut --fields=1 --delimiter=" " |
        xargs --no-run-if-empty kill -SIGUSR1
}

function set_background_theme() {
    local -r theme_path="${HOME}"/.local/share/backgrounds/backgrounds.dir
    set_theme_though_links "${theme_path}" "$@"
    set_background.sh
}

function set_bat_theme() {
    local -r theme_path="${HOME}"/.config/bat/config
    set_theme_though_links "${theme_path}" "$@"
}

function set_nvim_theme() {
    local -r theme_path="${HOME}"/.config/nvim/theme.txt
    set_theme_though_links "${theme_path}" "$@"
}

function set_rofi_theme() {
    local -r theme_path="${HOME}"/.config/rofi/theme.rasi
    set_theme_though_links "${theme_path}" "$@"
}

function set_gtk3_theme() {
    local -r theme_path="${HOME}"/.config/gtk-3.0/settings.ini
    set_theme_though_links "${theme_path}" "$@"
}

function set_local_theme() {
    local -r theme_path="${HOME}"/.config/theme/theme.txt
    set_theme_though_links "${theme_path}" "$@"
}

function set_theme_local() {
    dark_or_light="$1"

    set_local_theme "${dark_or_light}" &
    set_gtk3_theme "${dark_or_light}" &
    set_gtk4_theme "${dark_or_light}" &
    set_kitty_theme "${dark_or_light}" &
    set_background_theme "${dark_or_light}" &
    set_bat_theme "${dark_or_light}" &
    set_nvim_theme "${dark_or_light}" &
    set_rofi_theme "${dark_or_light}" &
    wait
}

function set_theme() {
    # ssh localhost -- "$(typeset -f); set_theme_local" "$@" &
    set_theme_local "$@"
}

function get_theme() {
    local -r theme_file="${HOME}"/.config/theme/theme.txt
    [[ -f ${theme_file} ]] && cat "${theme_file}"
}

function toggle_theme() {
    theme="$(get_theme)"
    if [[ ${theme} == "dark" ]]; then
        set_theme "light"
    else
        set_theme "dark"
    fi
}

function main() {
    if [[ $# -ne 1 ]]; then
        help_and_exit
    fi
    theme="$1"
    case "${theme}" in
    light)
        set_theme "light"
        ;;
    dark)
        set_theme "dark"
        ;;
    toggle)
        toggle_theme
        ;;
    *)
        help_and_exit
        ;;
    esac

}

if [[ $# -ne 1 ]] || [[ ${1} != "--source-only" ]]; then
    main "${@}"
fi
