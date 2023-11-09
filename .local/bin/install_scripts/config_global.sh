#!/usr/bin/env bash

function check_global_configs_files() {
    set -euo pipefail
    diff <(
        find "${HOME}"/.config/global_configs -type f |
            sed --regexp-extended "s|^${HOME}/||" | sort
    ) <(
        git ls-tree --full-tree --name-only -r HEAD "${HOME}"/.config/global_configs |
            sort
    ) || (
        echo "Untracked configs in ~/.config/global_configs" &&
            exit 1
    )
}

function copy_global_configs() {
    set -euo pipefail
    check_global_configs_files
    sudo cp --backup=numbered --verbose --recursive "${HOME}"/.config/global_configs/* /
}

function fix_keychron_post_config_copy() {
    sudo update-initramfs -u
}

function regenerate_grub_post_config_copy() {
    sudo update-grub
}

function reconfigure_tlp_post_config_copy() {
    sudo systemctl restart tlp
}

function todo_post_global_configs_copy() {
    fix_keychron_post_config_copy
    regenerate_grub_post_config_copy
    reconfigure_tlp_post_config_copy
}

function add_user_to_groups() {
    for group in docker input kvm lpadmin audio netdev video libvirt; do
        getent group "${group}" &&
            sudo usermod -aG "${group}" "${USER}" ||
            echo "Adding to ${group} failed"
    done
}

function enable_32_bit_architecture() {
    sudo dpkg --add-architecture i386
}

function update_locales() {
    echo "locales locales/default_environment_locale select en_US.UTF-8" |
        sudo debconf-set-selections
    echo "locales locales/locales_to_be_generated multiselect en_US.UTF-8 UTF-8, pl_PL.UTF-8 UTF-8" |
        sudo debconf-set-selections
    sudo rm "/etc/locale.gen"
    sudo dpkg-reconfigure --frontend noninteractive locales
}

function install_and_config_ssh_server() {
    if [[ -f "${HOME}/.local/bin/.config_sshd" ]]; then
        sudo apt-get update --yes &&
            sudo apt-get install --yes openssh-server
        "${HOME}"/.local/bin/.config_sshd
    fi
}

function create_global_set_display_script() {
    sudo cp ~/.local/bin/set_display /usr/bin/
}

function fix_redshift() {
    # Disable problematic redshift autostart
    sudo rm --force /etc/systemd/user/default.target.wants/redshift*.service
}

function configure_newt_palette() {
    # Set the original palette as a default option
    update-alternatives --query newt-palette &&
        sudo update-alternatives --set newt-palette /etc/newt/palette.original ||
        echo "no newt-palette"
}

function create_swap_file() {
    local -r swapfile_location="/home/swapfile"
    if ! [[ -f "${swapfile_location}" ]]; then
        sudo fallocate -l 2G "${swapfile_location}"
        sudo chmod 600 "${swapfile_location}"
        sudo mkswap "${swapfile_location}"
        sudo swapon "${swapfile_location}"
        echo "${swapfile_location} none    swap    sw    0   0" |
            sudo tee -a /etc/fstab
    fi
}
function configure_touchpad() {
    local -r config_file="/usr/share/X11/xorg.conf.d/40-libinput.conf"
    # Tap to click
    grep --quiet 'Option "Tapping" "on"' "${config_file}" ||
        sudo sed 's/^\(\( *\)Identifier "[a-zA-Z0-9 ]*touchpad[a-zA-Z0-9 ]*" *\)$/\1\n\2Option "Tapping" "on"/' -i "${config_file}"
}
function config_global_start() {
    copy_global_configs
    update_locales
    enable_32_bit_architecture
}

function config_global_rest() {
    todo_post_global_configs_copy
    add_user_to_groups
    configure_newt_palette
    configure_touchpad
    create_global_set_display_script
    create_swap_file
    fix_redshift
    install_and_config_ssh_server
}

function config_global_main() {
    set -euo pipefail
    set -x

    config_global_start
    config_global_rest
}

if [[ "$#" -ne 1 ]] || [[ "${1}" != "--source-only" ]]; then
    config_global_main "${@}"
fi
