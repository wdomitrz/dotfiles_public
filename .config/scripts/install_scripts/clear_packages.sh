#!/usr/bin/env bash
set -euo pipefail

function uninstall_packages_and_cleanup() {
    xargs sudo apt-get autoremove --purge --yes <"${HOME}/.config/packages/to_remove.txt" ||
        true
}

function main() {
    set -x

    uninstall_packages_and_cleanup
}

if [[ "$#" -ne 1 ]] || [[ "${1}" != "--source-only" ]]; then
    main "${@}"
fi
