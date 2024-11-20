#!/usr/bin/env bash

function install_deb_from_url() {
    link="$1"
    package_file="$(mktemp /tmp/XXXXXX.deb)"
    wget "${link}" -O "${package_file}"
    sudo apt-get install --yes "${package_file}"
    rm "${package_file}"
}

function install_gcloud() {
    echo "deb [signed-by=/usr/share/keyrings/cloud.google.gpg] https://packages.cloud.google.com/apt cloud-sdk main" |
        sudo tee /etc/apt/sources.list.d/google-cloud-sdk.list
    curl https://packages.cloud.google.com/apt/doc/apt-key.gpg |
        sudo apt-key --keyring /usr/share/keyrings/cloud.google.gpg add -
    sudo apt-get update --yes &&
        sudo apt-get install --yes google-cloud-sdk
}

function install_signal() {
    # 1. Install our official public software signing key
    wget -O- https://updates.signal.org/desktop/apt/keys.asc |
        gpg --dearmor |
        sudo tee /usr/share/keyrings/signal-desktop-keyring.gpg > /dev/null

    # 2. Add our repository to your list of repositories
    echo 'deb [arch=amd64 signed-by=/usr/share/keyrings/signal-desktop-keyring.gpg] https://updates.signal.org/desktop/apt xenial main' |
        sudo tee /etc/apt/sources.list.d/signal-xenial.list

    # 3. Update your package database and install signal
    sudo apt-get update --yes &&
        sudo apt-get install --yes signal-desktop
}

function add_google_public_key() {
    wget -q -O - https://dl.google.com/linux/linux_signing_key.pub |
        sudo apt-key add -

}

function install_google_chrome() {
    install_deb_from_url "https://dl.google.com/linux/direct/google-chrome-stable_current_amd64.deb"
}

function install_vscode() {
    install_deb_from_url "https://code.visualstudio.com/sha/download?build=stable&os=linux-deb-x64"
}

function install_chrome_remote_desktop() {
    install_deb_from_url "https://dl.google.com/linux/direct/chrome-remote-desktop_current_amd64.deb"
    # Configure Chrome Remote Desktop
    echo "Configure Chrome Remote Desktop at" "https://remotedesktop.google.com/headless"
}

function install_slack() {
    install_deb_from_url "https://downloads.slack-edge.com/releases/linux/4.26.1/prod/x64/slack-desktop-4.26.1-amd64.deb"
}

function install_nordvpn() {
    install_deb_from_url "https://repo.nordvpn.com/deb/nordvpn/debian/pool/main/nordvpn-release_1.0.0_all.deb"
    sudo apt-get update --yes
    sudo apt-get install --yes nordvpn
}

function not_sudo() {
    "$@"
}

function install_nvim_tar_given_locations() {
    # appimage url: https://github.com/neovim/neovim/releases/download/stable/nvim.appimage
    if [[ "$#" -ne 3 ]]; then
        echo "expected 2 parameters: <save_dir> <link_dir> <sudo_or_not_sudo>"
        return 1
    fi
    local -r save_dir="$1" link_dir="$2" sudo_or_not_sudo="$3"

    "${sudo_or_not_sudo}" mkdir --parents "${save_dir}" "${link_dir}"

    wget -qO- https://github.com/neovim/neovim/releases/download/stable/nvim-linux64.tar.gz | "${sudo_or_not_sudo}" tar xvz -C "${save_dir}"

    "${sudo_or_not_sudo}" ln --symbolic --relative --force "${save_dir}"/nvim-linux64/bin/nvim "${link_dir}"/
}

function install_nvim_tar_as_system_nvim() {
    install_nvim_tar_given_locations /opt /usr/bin sudo
}

function install_tailscale() {
    curl -fsSL https://tailscale.com/install.sh | sh
}

function install_dust() {
    install_deb_from_url "https://github.com/bootandy/dust/releases/download/v1.1.1/du-dust_1.1.1-1_amd64.deb"
}

function install_usb_c_display_driver() {
    # Synaptics displaylink-driver
    install_deb_from_url "https://www.synaptics.com/sites/default/files/Ubuntu/pool/stable/main/all/synaptics-repository-keyring.deb"
    sudo apt-get update --yes
    sudo apt-get install --yes displaylink-driver evdi-dkms
}

function install_global_main() {
    set -euo pipefail
    set -x
    source "${HOME}"/.local/bin/install_scripts/install_packages.sh --source-only

    install_google_chrome
    install_nordvpn
    install_signal
    install_tailscale
    install_vscode
    install_nvim_tar_as_system_nvim
    install_dust
    install_usb_c_display_driver
    update_and_upgrade
}

if [[ "$#" -ne 1 ]] || [[ "${1}" != "--source-only" ]]; then
    install_global_main "${@}"
fi
