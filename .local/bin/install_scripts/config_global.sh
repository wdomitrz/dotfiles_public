#!/usr/bin/env bash
set -uo pipefail

function check_files() {
    set -euo pipefail
    where="$1"
    diff <(
        find "${HOME}"/.config/"${where}" -type f |
            sed -E "s|^${HOME}/||" | sort
    ) <(
        git ls-tree --full-tree --name-only -r HEAD "${HOME}"/.config/"${where}" |
            sort
    ) || (
        echo "Untracked configs in ~/.config/${directory}" &&
            exit 1
    )
}

function copy_global_configs() {
    set -euo pipefail
    for directory in boot etc lib; do
        check_files "${directory}"
        sudo cp --verbose --recursive "${HOME}"/.config/"${directory}"/* /"${directory}"/
    done
}

function reconfigure_tlp_post_config_copy() {
    sudo systemctl restart tlp
}

function fix_keychron_post_config_copy() {
    sudo update-initramfs -u
}

function regenerate_grub_post_config_copy() {
    sudo update-grub
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
    echo "locales locales/locales_to_be_generated multiselect en_US.UTF-8 UTF-8, en_GB.UTF-8 UTF-8, pl_PL.UTF-8 UTF-8" |
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
    sudo rm -f /etc/systemd/user/default.target.wants/redshift*.service
}

function configure_bluetooth() {
    ## Enable user space HID
    sudo sed -i -E "s/^#?\s?(UserspaceHID)\s*=\s*[a-z]*$/\1=true/g" /etc/bluetooth/input.conf
    ## Set idle timeout
    sudo sed -i -E "s/^#?\s?(IdleTimeout\s*=\s*[0-9]*)$/\1/g" /etc/bluetooth/input.conf
    # fast bluetooth config
    ## Enable FastConnect
    sudo sed -i -E "s/^#?(FastConnectable)\s*=\s*[a-z]*/\1=true/g" /etc/bluetooth/main.conf
    ## Set number of reconnect attempts
    sudo sed -i -E "s/^#?(ReconnectAttempts\s*=\s*[0-9]*)$/\1/g" /etc/bluetooth/main.conf
    ## Set reconnect intervals
    sudo sed -i -E "s/^#?(ReconnectIntervals\s*=\s*[0-9, ]*)$/\1/g" /etc/bluetooth/main.conf
}

function configure_newt_palette() {
    # Set the original palette as a default option
    update-alternatives --query newt-palette &&
        sudo update-alternatives --set newt-palette /etc/newt/palette.original ||
        echo "no newt-palette"
}

function create_swap_file() {
    SWAPFILE_LOCATION="/home/swapfile"
    if ! [[ -f "${SWAPFILE_LOCATION}" ]]; then
        sudo fallocate -l 2G "${SWAPFILE_LOCATION}"
        sudo chmod 600 "${SWAPFILE_LOCATION}"
        sudo mkswap "${SWAPFILE_LOCATION}"
        sudo swapon "${SWAPFILE_LOCATION}"
        echo "${SWAPFILE_LOCATION} none    swap    sw    0   0" |
            sudo tee -a /etc/fstab
    fi
}
function configure_touchpad() {
    config_file="/usr/share/X11/xorg.conf.d/40-libinput.conf"
    # Tap to click
    grep --quiet 'Option "Tapping" "on"' "${config_file}" ||
        sudo sed 's/^\(\( *\)Identifier "[a-zA-Z0-9 ]*touchpad[a-zA-Z0-9 ]*" *\)$/\1\n\2Option "Tapping" "on"/' -i "${config_file}"
}

function main() {
    set -e
    set -x

    copy_global_configs
    todo_post_global_configs_copy

    add_user_to_groups
    configure_bluetooth
    configure_newt_palette
    configure_touchpad
    create_global_set_display_script
    create_swap_file
    fix_redshift
    install_and_config_ssh_server
    update_locales
}

if [[ "$#" -ne 1 ]] || [[ "${1}" != "--source-only" ]]; then
    main "${@}"
fi
