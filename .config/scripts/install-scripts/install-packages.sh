#!/usr/bin/env bash

function update_and_upgrade {
    sudo apt-get update --yes &&
        sudo apt-get dist-upgrade --yes
}

function install_packages {
    xargs sudo apt-get install --yes --no-install-recommends < \
        "$HOME/.config/packages/packages.txt"
}

function install_with_recommended {
    xargs sudo apt-get install --yes --install-recommends < \
        "$HOME/.config/packages/with-recommends.txt"
}

function install_wallpapers {
    sudo apt-get install --yes \
        ubuntu-wallpapers*
    sudo apt-mark auto ubuntu-wallpapers-*
    sudo apt-get autoremove --purge --yes
}

function install_restricted_extras {
    sudo apt-get install --yes \
        ubuntu-restricted-extras
}

function main {
    set -xue
    source "$HOME"/.config/scripts/install-scripts/config-global.sh --source-only

    configure_apt
    enable_32_bit_architecture
    update_and_upgrade
    install_packages
    install_with_recommended
    install_wallpapers
    install_restricted_extras
}

if [ "$#" -ne 1 ] || [ "${1}" != "--source-only" ]; then
    main "${@}"
fi
