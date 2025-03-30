#!/usr/bin/env bash

function install_deb_from_url() {
    link="$1"
    package_file="$(mktemp /tmp/XXXXXX.deb)"
    wget_with_defaults.sh --max-redirect=1 "${link}" > "${package_file}"
    sudo apt-get install --yes --no-install-recommends "${package_file}"
    rm "${package_file}"
}

function enable_source_and_list_packages() {
    set -euo pipefail
    sources_file=/etc/apt/sources.list.d/"$1"

    if grep --quiet '^Enabled: yes$' "${sources_file}"; then
        echo 1>&2 "${sources_file} already enabled"
        return 0
    fi
    if grep --quiet '^Signed-By:$' "${sources_file}"; then
        echo 1>&2 "${sources_file} already has Signed-By section. Invalid format."
        return 1
    fi

    key_source=$(grep 'key_source: ' "${sources_file}" | sed 's/key_source: //g')
    (
        echo "Signed-By:"
        wget_with_defaults.sh "${key_source}" | sponge |
            sed --expression='s|^$|.|g' --expression='s|^| |g'
    ) | sudo sponge -a "${sources_file}"

    sudo sed --in-place 's/^Enabled: no$/Enabled: yes/g' "${sources_file}"

    (grep '^packages_to_install: ' "${sources_file}" || true) |
        sed 's/packages_to_install: //g'
}

function enable_sources_and_install_packages() {
    packages_to_install=""
    while [[ $# -gt 0 ]]; do
        packages_from_this_source="$(enable_source_and_list_packages "$1")"
        packages_to_install="${packages_to_install} ${packages_from_this_source}"
        shift 1
    done
    read -r -a packages_to_install <<< "${packages_to_install}"

    sudo apt-get update --yes &&
        sudo apt-get install --yes --no-install-recommends "${packages_to_install[@]}"
}

function install_packages_external() {
    enable_sources_and_install_packages \
        google_chrome.sources \
        nordvpn.sources \
        signal.sources \
        tailscale.sources \
        vscode.sources
}

function install_chrome_remote_desktop() {
    enable_sources_and_install_packages chrome_remote_desktop.sources
    echo "Configure Chrome Remote Desktop at" "https://remotedesktop.google.com/headless"
}

function not_sudo() {
    "$@"
}

function install_to_given_location() {
    if [[ $# -ne 6 ]]; then
        echo "Usage: $0 <save_dir> <link_dir> <sudo_or_not_sudo> <url> <relative_binary_path> <decompression_option>"
        return 1
    fi
    local -r save_dir="$1" link_dir="$2" sudo_or_not_sudo="$3" url="$4" relative_binary_path="$5" decompression_option="$6"

    "${sudo_or_not_sudo}" mkdir --parents "${save_dir}" "${link_dir}"

    wget_with_defaults.sh --max-redirect=1 "${url}" |
        "${sudo_or_not_sudo}" tar --extract \
            "${decompression_option}" \
            --directory="${save_dir}"

    "${sudo_or_not_sudo}" ln --symbolic --relative --force "${save_dir}"/"${relative_binary_path}" "${link_dir}"/
}

function install_nvim_tar_given_locations() {
    # appimage url: https://github.com/neovim/neovim/releases/download/stable/nvim.appimage
    [[ $# -ne 3 ]] && exit 1
    local -r save_dir="$1" link_dir="$2" sudo_or_not_sudo="$3"
    install_to_given_location "${save_dir}" "${link_dir}" "${sudo_or_not_sudo}" \
        "https://github.com/neovim/neovim/releases/download/stable/nvim-linux-x86_64.tar.gz" \
        nvim-linux-x86_64/bin/nvim \
        --ungzip
}

function install_nvim_tar_as_system_nvim() {
    install_nvim_tar_given_locations /opt /usr/bin sudo
}

function install_dust() {
    install_deb_from_url "https://github.com/bootandy/dust/releases/download/v1.1.1/du-dust_1.1.1-1_amd64.deb"
}

function install_usb_c_display_driver() {
    install_deb_from_url https://www.synaptics.com/sites/default/files/Ubuntu/pool/stable/main/all/synaptics-repository-keyring.deb
    sudo apt-get update --yes &&
        sudo apt-get install --yes --no-install-recommends displaylink-driver evdi-dkms
}

function install_global_main() {
    set -euo pipefail
    set -x
    source "${HOME}"/.local/bin/install_scripts/install_packages.sh --source-only

    install_packages_external
    install_dust
    update_and_upgrade
}

if [[ $# -ne 1 ]] || [[ ${1} != "--source-only" ]]; then
    install_global_main "${@}"
fi
