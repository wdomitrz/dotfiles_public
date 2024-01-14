#!/usr/bin/env bash

function update_and_upgrade() {
    sudo apt-get update --yes &&
        sudo apt-get dist-upgrade --yes
}

function install_packages() {
    packages_file="${HOME}"/.config/packages/packages.sorted.txt
    xargs sudo DEBIAN_FRONTEND=noninteractive apt-get install --yes --no-install-recommends <"${packages_file}"
}

function install_nvidia() {
    if ! (lspci | grep --quiet --ignore-case nvidia); then
        echo "Not installing nvidia driver, becuase it's not needed"
        return 0
    fi
    sudo DEBIAN_FRONTEND=noninteractive apt-get install --yes --install-recommends nvidia-driver
}

function install_with_recommended() {
    packages_file="${HOME}"/.config/packages/with_recommends.sorted.txt
    xargs sudo DEBIAN_FRONTEND=noninteractive apt-get install --yes --install-recommends <"${packages_file}"
}

function install_packages_main() {
    set -euo pipefail
    set -x
    source "${HOME}"/.local/bin/install_scripts/config_global.sh --source-only

    config_global_start
    update_and_upgrade
    install_packages
    install_nvidia
    install_with_recommended
}

if [[ "$#" -ne 1 ]] || [[ "${1}" != "--source-only" ]]; then
    install_packages_main "${@}"
fi
