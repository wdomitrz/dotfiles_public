#!/usr/bin/env bash

function uninstall_packages_and_cleanup() {
    xargs sudo apt-get autoremove --purge --yes <"${HOME}"/.config/packages/to_remove.sorted.txt ||
        true
}

function clear_packages_main() {
    set -euo pipefail
    set -x

    uninstall_packages_and_cleanup
}

if [[ "$#" -ne 1 ]] || [[ "${1}" != "--source-only" ]]; then
    clear_packages_main "${@}"
fi
