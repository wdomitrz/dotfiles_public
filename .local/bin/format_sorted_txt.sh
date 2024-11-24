#!/usr/bin/env bash

function format_function() {
    [[ "$#" -eq 1 ]] || return 1
    sort --output "$1"{,}
}
export -f format_function

echo "$@" | exec xargs --max-args=1 bash -c 'format_function "$@"' --
