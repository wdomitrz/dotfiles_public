#!/usr/bin/env bash

function uninstall_packages_and_cleanup() {
    xargs sudo apt-get autoremove --purge --yes <"$HOME/.config/packages/to-remove.txt" ||
        true
}

function main() {
    set -xue

    uninstall_packages_and_cleanup
}

if [ "$#" -ne 1 ] || [ "${1}" != "--source-only" ]; then
    main "${@}"
fi
