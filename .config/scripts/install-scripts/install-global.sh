#!/usr/bin/env bash

function install_deb_from_url {
    link="$1"
    package_file="$(mktemp /tmp/XXXXXX.deb)"
    wget "${link}" -O "${package_file}"
    sudo apt-get install --yes "${package_file}"
    rm "${package_file}"
}

function install_gcloud {
    echo "deb [signed-by=/usr/share/keyrings/cloud.google.gpg] https://packages.cloud.google.com/apt cloud-sdk main" | sudo tee /etc/apt/sources.list.d/google-cloud-sdk.list
    curl https://packages.cloud.google.com/apt/doc/apt-key.gpg | sudo apt-key --keyring /usr/share/keyrings/cloud.google.gpg add -
    sudo apt-get update --yes && sudo apt-get install --yes google-cloud-sdk
}

function main {
    set -xue

    # Google Chrome
    install_deb_from_url "https://dl.google.com/linux/direct/google-chrome-stable_current_amd64.deb"
    # Add google public key
    wget -q -O - https://dl-ssl.google.com/linux/linux_signing_key.pub | sudo apt-key add -

    # VSCode
    install_deb_from_url "https://code.visualstudio.com/sha/download?build=stable&os=linux-deb-x64"

    # Google Cloud SDK
    install_gcloud

    # Check for updates and upgrades
    sudo apt-get update --yes && sudo apt-get dist-upgrade --yes
}

if [ "$#" -ne 1 ] || [ "${1}" != "--source-only" ]; then
    main "${@}"
fi
