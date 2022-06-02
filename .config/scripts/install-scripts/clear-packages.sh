#!/usr/bin/env bash

function uninstall_packages_and_cleanup {
    cat "$HOME/.config/packages/to-remove.txt" |
        sudo apt-get autoremove --purge --yes
}

function main {
    set -xue

    uninstall_packages_and_cleanup
}

if [ "$#" -ne 1 ] || [ "${1}" != "--source-only" ]; then
    main "${@}"
fi
