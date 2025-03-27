#!/usr/bin/env bash

function update_and_upgrade() {
    sudo apt-get update --yes &&
        sudo apt-get dist-upgrade --yes
}

function install_packages_from() {
    packages_file="$1"
    xargs sudo DEBIAN_FRONTEND=noninteractive apt-get install --yes --no-install-recommends < "${packages_file}"
}

function install_packages_base() {
    install_packages_from "${HOME}"/.config/packages/packages_base.sorted.txt
}

function install_packages_rest() {
    install_packages_from "${HOME}"/.config/packages/packages_rest.sorted.txt
}

function install_nvidia() {
    if ! (lspci | grep --quiet --ignore-case nvidia); then
        echo "Not installing nvidia driver, because it's not needed"
        return 0
    fi
    sudo DEBIAN_FRONTEND=noninteractive apt-get install --yes --install-recommends nvidia-driver
}

function install_packages_main() {
    set -euo pipefail
    set -x
    source "${HOME}"/.local/bin/install_scripts/config_global.sh --source-only

    update_and_upgrade
    install_packages_base
    config_global_start
    update_and_upgrade
    install_packages_rest
    install_nvidia
}

if [[ $# -ne 1 ]] || [[ ${1} != "--source-only" ]]; then
    install_packages_main "${@}"
fi
