#!/usr/bin/env bash

function update_and_upgrade() {
    sudo apt-get update --yes &&
        sudo apt-get dist-upgrade --yes
}

function install_packages() {
    packages_file="$HOME/.config/packages/packages.txt"
    xargs sudo DEBIAN_FRONTEND=noninteractive apt-get install --yes --no-install-recommends <"$packages_file"
}

function install_with_recommended() {
    packages_file="$HOME/.config/packages/with-recommends.txt"
    xargs sudo DEBIAN_FRONTEND=noninteractive apt-get install --yes --install-recommends <"$packages_file"
}

function main() {
    set -xue
    source "$HOME"/.config/scripts/install-scripts/config-global.sh --source-only

    update_locales
    configure_debian_sources_list
    configure_apt
    enable_32_bit_architecture
    update_and_upgrade
    install_packages
    install_with_recommended
}

if [ "$#" -ne 1 ] || [ "${1}" != "--source-only" ]; then
    main "${@}"
fi
