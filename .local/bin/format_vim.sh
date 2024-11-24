#!/usr/bin/env bash

function format_function() {
    set -ue
    [[ "$#" -eq 1 ]] || return 1
    nvim --noplugin --headless \
        -c 'silent norm gg=G' \
        -c 'silent wqa' \
        -- "$@" \
        2> /dev/null
}
export -f format_function

echo "$@" | exec xargs --max-args=1 bash -c 'format_function "$@"' --
