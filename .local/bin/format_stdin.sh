#!/usr/bin/env bash
function format_json() { jq --indent 4 . -; }

function format_py() { ruff format -; }

function format_sh() { shfmt --indent 4 --space-redirects --simplify -; }

function format_sorted_json() { jq --indent 4 --sort-keys . -; }

function format_sorted_numeric_txt() { sort --numeric-sort -; }

function format_sorted_txt() { sort -; }

function format_vim() {
    tmp_file="$(mktemp --suffix=.vim)"
    cat > "${tmp_file}"

    nvim --headless \
        -c 'silent norm gg=G' \
        -c 'silent wqa' \
        -- "${tmp_file}" \
        2> /dev/null

    cat "${tmp_file}"
    rm --force "${tmp_file}"
}

function format_stdin_main() {
    set -euo pipefail
    if [[ $# -ne 1 ]]; then
        echo "${FUNCNAME[*]}: Expected exactly one argument - file type"
        return 1
    fi
    case "$1" in
    *.sh | sh | bash) format_sh ;;
    *.py | py | python) format_py ;;
    *.sorted.json | sorted_json) format_sorted_json ;;
    *.json | json) format_json ;;
    *.vim | vim) format_vim ;;
    *.sorted.txt | sorted_txt | sorted) format_sorted_txt ;;
    *.sorted_numeric.txt | sorted_numeric_txt) format_sorted_numeric_txt ;;
    *) cat ;;
    esac
}

if [[ $# -ne 1 ]] || [[ ${1} != "--source-only" ]]; then
    LC_ALL=C format_stdin_main "${@}"
fi
