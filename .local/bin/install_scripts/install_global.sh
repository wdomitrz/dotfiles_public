#!/usr/bin/env bash

function install_deb_from_url() {
    link="$1"
    package_file="$(mktemp /tmp/XXXXXX.deb)"
    wget_with_defaults.sh --max-redirect=1 "${link}" > "${package_file}"
    sudo apt-get install --yes "${package_file}"
    rm "${package_file}"
}

function enable_sources_file() {
    set -euo pipefail
    sources_file=/etc/apt/sources.list.d/"$1"

    key_source=$(grep 'Key-Source: ' "${sources_file}" | sed 's/Key-Source: //g')
    key_file=$(grep 'Signed-By: ' "${sources_file}" | sed 's/Signed-By: //g')

    sudo mkdir --parents "$(dirname "${key_file}")"
    wget_with_defaults.sh "${key_source}" | sudo sponge "${key_file}"

    sudo sed --in-place 's/^Enabled: no$/Enabled: yes/g' "${sources_file}"
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

    wget_with_defaults.sh --max-redirect=1 https://github.com/neovim/neovim/releases/download/stable/nvim-linux-x86_64.tar.gz |
        "${sudo_or_not_sudo}" tar xvz -C "${save_dir}"

    "${sudo_or_not_sudo}" ln --symbolic --relative --force "${save_dir}"/nvim-linux-x86_64/bin/nvim "${link_dir}"/
}

function install_nvim_tar_as_system_nvim() {
    install_nvim_tar_given_locations /opt /usr/bin sudo
}

function install_dust() {
    install_deb_from_url "https://github.com/bootandy/dust/releases/download/v1.1.1/du-dust_1.1.1-1_amd64.deb"
}

function install_chrome_remote_desktop() {
    enable_sources_file chrome_remote_desktop.sources
    sudo apt-get update --yes &&
        sudo apt-get install --yes chrome-remote-desktop
    # Configure Chrome Remote Desktop
    echo "Configure Chrome Remote Desktop at" "https://remotedesktop.google.com/headless"
}

function install_usb_c_display_driver() {
    install_deb_from_url https://www.synaptics.com/sites/default/files/Ubuntu/pool/stable/main/all/synaptics-repository-keyring.deb
    sudo apt-get update --yes &&
        sudo apt-get install --yes displaylink-driver evdi-dkms
}

function install_packages_external() {
    enable_sources_file google_chrome.sources
    enable_sources_file signal.sources
    enable_sources_file tailscale.sources
    enable_sources_file vscode.sources

    source "${HOME}"/.local/bin/install_scripts/install_packages.sh --source-only &&
        sudo apt-get update --yes &&
        install_packages_from "${HOME}"/.config/packages/packages_external.sorted.txt
}

function install_global_main() {
    set -euo pipefail
    set -x
    source "${HOME}"/.local/bin/install_scripts/install_packages.sh --source-only

    install_packages_external
    install_nvim_tar_as_system_nvim
    install_dust
    update_and_upgrade
}

if [[ "$#" -ne 1 ]] || [[ "${1}" != "--source-only" ]]; then
    install_global_main "${@}"
fi
