#!/usr/bin/env bash
function get_list_of_modifiers() {
    xmodmap -pm | tail -n +2 | cut --delimiter=" " --fields=1 | sed -r '/^$/d'
}

function validate_modifier() {
    if [[ "$#" -ne 1 ]]; then
        return 1
    fi
    get_list_of_modifiers | grep --quiet '^'"$1"'$'
}

function clear_all_options() {
    setxkbmap -option
    pkill -9 --full xcape || true
    xmodmap -e "clear lock"
}

function set_as_tap_hold() {
    if [[ "$#" -ne 5 ]]; then
        echo "\
usage: ${FUNCNAME[0]} <keycode_to_use> <key_on_tap> <modifier_on_hold> <spare_key> <modifier_on_spare_modifier>

List of modifiers:
$ xmodmap -pm

Spare modifier keys:
Hyper_L
"
        return 1
    fi

    keycode_to_use="$1"
    key_on_tap="$2"
    modifier_on_hold="$3"
    spare_modifier="$4"
    modifier_on_spare_modifier="$5"

    if ! validate_modifier "${modifier_on_hold}"; then
        echo "Invalid modifier ${modifier_on_hold}. "
    fi

    xmodmap -e "keycode ${keycode_to_use} = ${spare_modifier}"
    # Remove default modifier
    xmodmap -e "remove ${modifier_on_spare_modifier} = ${spare_modifier}"
    xmodmap -e "add ${modifier_on_hold} = ${spare_modifier}"
    # In case we unmapped this key
    xmodmap -e "keycode any = ${key_on_tap}"
    xcape -e "${spare_modifier}=${key_on_tap}"
}

function set_keymap_base() {
    clear_all_options

    tab_keycode=23 space_keycode=65 caps_lock_keycode=66
    set_as_tap_hold "${tab_keycode}" Tab mod1 Hyper_L mod4
    set_as_tap_hold "${space_keycode}" space mod4 Super_R mod4
    set_as_tap_hold "${caps_lock_keycode}" Escape control Hyper_R mod4
}

function set_keymap_main() {
    set_keymap_base
    sleep 2
    set_keymap_base
    sleep 5
    set_keymap_base
}

if [[ "$#" -ne 1 ]] || [[ "${1}" != "--source-only" ]]; then
    set_keymap_main "${@}"
fi
