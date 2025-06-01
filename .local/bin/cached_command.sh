#!/usr/bin/env bash
set -euo pipefail

function usage_and_exit() {
    echo "usage: $(basename "$0") [-h|--help] [--overwrite] <command> [<command_params>]"
    exit "$1"
}

overwrite=false
case "$1" in
-h | --help)
    usage_and_exit 0
    ;;
--overwrite)
    overwrite=true
    shift 1
    ;;
*) ;;
esac

if [[ $# -eq 0 ]]; then
    usage_and_exit 1
fi

base_cache_dir="/run/user/$(id -u "${USER}")"

if [[ ! -d ${base_cache_dir} ]]; then exec "$@"; fi

args_hash="$(printf '%s\t\0' "$@" | md5sum | awk '{print $1}')"
cache_file="${base_cache_dir}/cached_command/${args_hash}.txt"

if "${overwrite}" || [[ ! -f ${cache_file} ]]; then
    # shellcheck disable=SC2174
    mkdir --parents --mode=700 "$(dirname "${cache_file}")"

    touch "${cache_file}"
    chmod u=rw,g=,o= "${cache_file}"
    "$@" | sponge "${cache_file}"
fi
cat "${cache_file}"
