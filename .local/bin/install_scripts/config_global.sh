#!/usr/bin/env bash
set -uo pipefail

function check_files() {
    diff <(
        find "${HOME}"/.config/{boot,etc,lib} -type f |
            sed -E "s|^${HOME}/||" | sort
    ) <(
        git ls-tree --full-tree --name-only -r HEAD "${HOME}"/.config/{boot,etc,lib} |
            sort
    ) || (
        echo "Untracked configs"
        exit 1
    )
}

function add_user_to_groups() {
    for group in docker input kvm lpadmin audio netdev video libvirt; do
        getent group "${group}" &&
            sudo usermod -aG "${group}" "${USER}" ||
            echo "Adding to ${group} failed"
    done
}

function copy_configs_from_to() {
    local -r source_dir="$1"
    local -r target_dir="$2"
    if [[ ! -d "${source_dir}" ]]; then
        echo "No directory ${source_dir}"
        return 1
    fi
    sudo mkdir --parents "${target_dir}"
    sudo cp "${source_dir}"/* "${target_dir}"
}

function get_global_config() {
    local -r target_dir="$1"
    local -r source_dir="${HOME}"/.config/"${target_dir}"
    copy_configs_from_to "${source_dir}" "${target_dir}"
}

function enable_32_bit_architecture() {
    sudo dpkg --add-architecture i386
}

function configure_debian_sources_list() {
    grep --quiet "/deb.debian.org" /etc/apt/sources.list &&
        sudo cp "${HOME}"/.config/etc/apt/sources.list /etc/apt/sources.list ||
        echo "Not using debian"
}

function update_locales() {
    echo "locales locales/default_environment_locale select en_US.UTF-8" |
        sudo debconf-set-selections
    echo "locales locales/locales_to_be_generated multiselect en_US.UTF-8 UTF-8, en_GB.UTF-8 UTF-8, pl_PL.UTF-8 UTF-8" |
        sudo debconf-set-selections
    sudo rm "/etc/locale.gen"
    sudo dpkg-reconfigure --frontend noninteractive locales
}

function configure_apt() {
    get_global_config /etc/apt/apt.conf.d/
    get_global_config /etc/apt/preferences.d/
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

function fix_screen_tearing() {
    get_global_config /etc/X11/xorg.conf.d/
}

function set_global_keymap() {
    get_global_config /etc/default/
}

function configure_lightdm() {
    get_global_config /etc/lightdm/lightdm.conf.d/
    get_global_config /etc/lightdm/lightdm-gtk-greeter.conf.d/
}

function fix_network_manager() {
    get_global_config /etc/network/
    get_global_config /etc/NetworkManager/conf.d/
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

function fix_keychron() {
    # Keychron make Fn+f-keys work
    get_global_config /etc/modprobe.d/
    sudo update-initramfs -u
}

function configure_newt_palette() {
    # Set the original palette as a default option
    update-alternatives --query newt-palette &&
        sudo update-alternatives --set newt-palette /etc/newt/palette.original ||
        echo "no newt-palette"
}

function configure_grub() {
    get_global_config /boot/grub/
    get_global_config /etc/default/
    # Regenerate config
    sudo update-grub
}

function fix_iwlwifi() {
    get_global_config /etc/modprobe.d/iwlwifi.conf
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

    # Fix touchpad after sleep
    get_global_config /lib/systemd/system-sleep/
}

function configure_tlp() {
    # Enable charge threshold for tlp
    sudo sed -i 's/#\([A-Z]*_CHARGE_THRESH_BAT0=[0-9]*\)$/\1/' /etc/tlp.conf
    sudo systemctl restart tlp
}

function main() {
    set -e
    check_files
    set -x

    add_user_to_groups
    configure_apt
    configure_bluetooth
    configure_grub
    configure_lightdm
    configure_newt_palette
    configure_tlp
    configure_touchpad
    create_global_set_display_script
    create_swap_file
    fix_iwlwifi
    fix_keychron
    fix_network_manager
    fix_redshift
    fix_screen_tearing
    install_and_config_ssh_server
    set_global_keymap
    update_locales
}

if [[ "$#" -ne 1 ]] || [[ "${1}" != "--source-only" ]]; then
    main "${@}"
fi
