#!/usr/bin/env bash

function copy_global_configs() {
    check_integrity_of_tracked_dir.sh "${HOME}"/.config/global_configs &&
        sudo cp --update --backup=numbered --verbose --recursive \
            "${HOME}"/.config/global_configs/. /
}

function fix_keychron_post_config_copy() {
    sudo update-initramfs -u
}

function regenerate_grub_post_config_copy() {
    sudo update-grub
}

function reconfigure_tlp_post_config_copy() {
    sudo systemctl restart tlp || true
}

function todo_post_global_configs_copy() {
    fix_keychron_post_config_copy
    regenerate_grub_post_config_copy
    reconfigure_tlp_post_config_copy
}

function add_user_to_groups() {
    for group in sudo docker input uinput kvm lpadmin audio netdev video libvirt; do
        sudo groupadd "${group}" || true
        sudo usermod --append --groups "${group}" "${USER}" ||
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
    sudo rm --force --verbose "/etc/locale.gen"
    sudo dpkg-reconfigure --frontend noninteractive locales
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

function configure_external_power_sleep() {
    sudo sed --in-place=".bck" --regexp-extended "s/^#?(HandleLidSwitchExternalPower)\s*=\s*[a-z]*/\1=lock/g" /etc/systemd/logind.conf
    sudo sed --in-place=".bck" --regexp-extended "s/^#?(HandleLidSwitchDocked)\s*=\s*[a-z]*/\1=lock/g" /etc/systemd/logind.conf
}

function create_swap_file() {
    local -r swapfile_location="/home/swapfile"
    if ! [[ -f ${swapfile_location} ]]; then
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
        sudo sed --in-place 's/^\(\( *\)Identifier "[a-zA-Z0-9 ]*touchpad[a-zA-Z0-9 ]*" *\)$/\1\n\2Option "Tapping" "on"/' "${config_file}"
}

function udisk_allow_operations() {
    for operation in \
        '"org.freedesktop.udisks2.encrypted-unlock"' \
        '"org.freedesktop.udisks2.encrypted-unlock-system"' \
        '"org.freedesktop.udisks2.filesystem-mount"'; do
        for group in allow_any allow_inactive allow_active; do
            sudo sed -i \
                '/'"${operation}"'/,/'"${group}"'/{/'"${group}"'/{s/auth_admin[_keep]*/yes/;}}' \
                /usr/share/polkit-1/actions/org.freedesktop.UDisks2.policy
        done
    done
}

function nordvpn_with_tailscale() {
    nordvpn whitelist add subnet 100.64.0.0/10
    nordvpn whitelist add subnet fd7a:115c:a1e0::/48
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
    udisk_allow_operations
    configure_touchpad
    create_swap_file
    fix_redshift
}

function configure_tpm2_non_root_disk_unlock() {
    echo "Don't use this function" && exit 1

    # Add tpm2 as a method to open the drive on boot
    # For lines with encrypted disks (with `_crypt UUID`) add `,tpm2-device=auto`
    # to the last parameter
    sudo sed --in-place --expression='/_crypt UUID/s/[ \t]*$/,tpm2-device=auto/' /etc/crypttab
    sudo update-initramfs -u

    # Actually allow tmp2 to unlock the drive
    # replace UNSET_DEVICE with something like nvme0n1p3 (if above you had nvme0n1p3_crypt)
    # For info on slots:
    # sudo systemd-cryptenroll /dev/nvme0n1p3
    # For wiping unused slot
    # sudo systemd-cryptenroll --wipe-slot=1 /dev/nvme0n1p3
    sudo systemd-cryptenroll --tpm2-device=auto --tpm2-pcrs=7+8 /dev/nvme0n1p3
}

function config_global_main() {
    set -euo pipefail
    set -x

    # Only the part that is not used in install_packages
    config_global_rest
}

if [[ $# -ne 1 ]] || [[ ${1} != "--source-only" ]]; then
    config_global_main "${@}"
fi
