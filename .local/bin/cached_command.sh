#!/usr/bin/env bash
set -euo pipefail

function usage_and_exit() {
    echo "usage: $(basename "$0") [-h|--help] [--mtime_cache_too_old <mtime_cache_too_old>] <command> [<command_params>]"
    exit "$1"
}

if [[ "$#" -eq 0 ]]; then
    usage_and_exit 1
fi

mtime_cache_too_old="+1"
case "$1" in
-h | --help)
    usage_and_exit 0
    ;;
--mtime_cache_too_old)
    if [[ "$#" -lt 2 ]]; then
        usage_and_exit 1
    fi
    mtime_cache_too_old="$2"
    shift 2
    ;;
*) ;;
esac

args_hash="$(printf '%s\t\0' "$@" | md5sum | awk '{print $1}')"
cache_file="${HOME}/.cache/cached_command/cached_command_file_${args_hash}.txt"

if [[ ! -f "${cache_file}" ]] ||
    [[ -n "$(find "${cache_file}" -mtime "${mtime_cache_too_old}")" ]]; then
    # shellcheck disable=SC2174
    # --mode=700 applied only to the deepest directory - as intended
    mkdir --parents --mode=700 "$(dirname "${cache_file}")"

    touch "${cache_file}"
    chmod u=rw,g=,o= "${cache_file}"
    "$@" | sponge "${cache_file}"
fi
cat "${cache_file}"
