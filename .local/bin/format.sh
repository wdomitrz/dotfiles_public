#!/usr/bin/env bash

function format_json() {
    jq --indent 4 . -- "$@" | sponge "$@"
}
export -f format_json

function format_py() {
    ruff format --quiet "$@"
}
export -f format_py

function format_sh() {
    shfmt --indent 4 --space-redirects --write "$@"
}
export -f format_sh

function format_sorted_json() {
    jq --indent 4 --sort-keys . -- "$@" | sponge "$@"
}
export -f format_sorted_json

function format_sorted_numeric_txt() {
    sort --numeric-sort --output "$1"{,}
}
export -f format_sorted_numeric_txt

function format_sorted_txt() {
    sort --output "$1"{,}
}
export -f format_sorted_txt

function format_vim() {
    nvim --headless \
        -c 'silent norm gg=G' \
        -c 'silent wqa' \
        -- "$@" \
        2> /dev/null
}
export -f format_vim

function format_a_file() {
    [[ "$#" -eq 1 ]] || return 1
    file_path="$1"

    case ${file_path} in
    *.sh)
        format_sh "${file_path}"
        ;;
    *.py)
        format_py "${file_path}"
        ;;
    *.sorted.json)
        format_sorted_json.sh "${file_path}"
        ;;
    *.json)
        format_json "${file_path}"
        ;;
    *.vim)
        format_vim "${file_path}"
        ;;
    *.sorted.txt)
        format_sorted_txt.sh "${file_path}"
        ;;
    *.sorted_numeric.txt)
        format_sorted_numeric_txt.sh "${file_path}"
        ;;
    *) ;;
    esac
}
export -f format_a_file

function format_main() {
    set -euo pipefail
    echo "$@" | parallel format_a_file
}

if [[ "$#" -ne 1 ]] || [[ "${1}" != "--source-only" ]]; then
    format_main "${@}"
fi
