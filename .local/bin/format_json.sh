#!/usr/bin/env bash

function format_function() {
    [[ "$#" -eq 1 ]] || return 1
    jq --indent 4 . -- "$@" | sponge "$@"
}
export -f format_function

echo "$@" | exec xargs --max-args=1 bash -c 'format_function "$@"' --
