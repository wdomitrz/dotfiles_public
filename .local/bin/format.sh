#!/usr/bin/env bash
function format_a_file() {
    set -euo pipefail
    if [[ $# -eq 0 ]]; then
        echo "${FUNCNAME[*]}: Expected exactly one argument"
        return 1
    fi
    given_file_path="$1"

    resolved_file_path="$(realpath --quiet "${given_file_path}")" ||
        return 0 # Ignore dangling links
    if [[ -d ${resolved_file_path} ]] || [[ ! -w ${resolved_file_path} ]]; then
        return 0 # Ignore directories and non-writeable files
    fi

    # Only supported file types
    case "$1" in
    *.sh | *.py | *.sorted.json | *.json | *.vim | *.sorted.txt | *.sorted_numeric.txt) ;;
    *) return 0 ;;
    esac

    # shellcheck disable=SC2002
    cat "${given_file_path}" |
        format_stdin.sh "${given_file_path}" |
        sponge "${given_file_path}"
}
export -f format_a_file

function format_main() {
    set -euo pipefail
    if [[ $# -eq 0 ]]; then
        echo "Usage: $0 <file_path> [<file_path>]*"
        return 1
    fi
    find "$@" | LC_ALL=C exec parallel format_a_file
}

if [[ $# -ne 1 ]] || [[ ${1} != "--source-only" ]]; then
    format_main "${@}"
fi
