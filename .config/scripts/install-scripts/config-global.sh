#!/usr/bin/env bash

function add_user_to_groups {
    sudo usermod -aG docker,input,kvm,lpadmin,audio,netdev,video "$USER"
}

function copy_configs_from_to {
    source_dir="$1"
    target_dir="$2"
    if [ ! -d "$source_dir" ]; then
        echo "No directory $source_dir"
        return 1
    fi
    sudo mkdir -p "$target_dir"
    sudo cp "$source_dir"/* "$target_dir"
}

function enable_32_bit_architecture {
    sudo dpkg --add-architecture i386
}

function configure_debian_sources_list {
    grep --quiet "/deb.debian.org" /etc/apt/sources.list &&
        sudo cp "$HOME"/.config/debian/sources.list /etc/apt/sources.list ||
        echo "Not using debian"
}

function update_locales {
    echo "locales locales/default_environment_locale select en_US.UTF-8" | sudo debconf-set-selections
    echo "locales locales/locales_to_be_generated multiselect en_US.UTF-8 UTF-8, en_GB.UTF-8 UTF-8, pl_PL.UTF-8 UTF-8" | sudo debconf-set-selections
    sudo rm "/etc/locale.gen"
    sudo dpkg-reconfigure --frontend noninteractive locales
}

function configure_apt {
    copy_configs_from_to "$HOME"/.config/apt/apt.conf.d/ /etc/apt/apt.conf.d/
    copy_configs_from_to "$HOME"/.config/apt/preferences.d/ /etc/apt/preferences.d/
}

function install_and_config_ssh_server {
    if [ -f "$HOME/.local/bin/.config-sshd" ]; then
        sudo apt-get update --yes && sudo apt-get install --yes openssh-server
        "$HOME"/.local/bin/.config-sshd
    fi
}

function create_global_set_display_script {
    sudo cp ~/.local/bin/set-display /usr/bin/
}

function fix_screen_tearing {
    copy_configs_from_to "$HOME"/.config/xorg.conf.d/ /etc/X11/xorg.conf.d/
}

function set_global_keymap {
    # keybord options - map caps lock to ctrl
    sudo sed -i -E "s/^(XKBOPTIONS)\s*=\s*\"\"/\1=\"ctrl:nocaps\"/g" /etc/default/keyboard
}

function configure_lightdm {
    sudo sed -i -E "s/^(greeter-hide-users)\s*=\s*/#\1=/g" /usr/share/lightdm/*/*.conf
    copy_configs_from_to "$HOME"/.config/lightdm-gtk-greeter.conf.d/ /etc/lightdm/lightdm-gtk-greeter.conf.d/
    copy_configs_from_to "$HOME"/.config/lightdm.conf.d/ /etc/lightdm/lightdm.conf.d/
}

function fix_network_manager {
    sudo touch /etc/NetworkManager/conf.d/10-globally-managed-devices.conf
}

function get_certificates {
    copy_configs_from_to "$HOME"/.config/ca-certificates/ /usr/share/ca-certificates/
}

function fix_redshift {
    # Disable problematic redshift autostart
    sudo rm -f /etc/systemd/user/default.target.wants/redshift*.service
}

function configure_bluetooth {
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

function fix_keychron {
    # Keychron make Fn+f-keys work
    echo "options hid_apple fnmode=2" | sudo tee /etc/modprobe.d/hid_apple.conf
    sudo update-initramfs -u
}

function configure_newt_palette {
    # Set the original palette as a default option
    update-alternatives --query newt-palette &&
        sudo update-alternatives --set newt-palette /etc/newt/palette.original ||
        echo "no newt-palette"
}

function configure_grub {
    # Custom colors
    copy_configs_from_to "$HOME"/.config/grub/ /boot/grub/
    # Load saved option
    sudo sed -i -E "s/^(GRUB_DEFAULT)\s*=\s*[a-z0-9]+$/\1=saved/" /etc/default/grub
    # 1 second timeout
    sudo sed -i -E "s/^(GRUB_TIMEOUT)\s*=\s*[0-9]+$/\1=1/" /etc/default/grub
    # Enable OS prober
    grep --quiet "GRUB_DISABLE_OS_PROBER" /etc/default/grub &&
        sudo sed -i -E "s/^#?(GRUB_DISABLE_OS_PROBER)\s*=\s*[0-9a-z]+$/\1=false/" /etc/default/grub ||
        (echo -n "
# Enable OS prober
GRUB_DISABLE_OS_PROBER=false
" | sudo tee -a /etc/default/grub)
    # No splash screen
    sudo sed -i -E "s/ *splash//" /etc/default/grub
    # Regenerate config
    sudo update-grub
}

function fix_iwlwifi {
    # Fix iwlwifi iwl-debug-yoyo.bin error
    FIX="options iwlwifi enable_ini=N"
    CONF="/etc/modprobe.d/iwlwifi.conf"
    if ! grep -s "$FIX" "$CONF" >/dev/null; then
        echo "$FIX" | sudo tee -a "$CONF"
    fi
}

function create_swap_file {
    SWAPFILE_LOCATION="/swapfile"
    if ! [ -f "${SWAPFILE_LOCATION}" ]; then
        sudo fallocate -l 2G "${SWAPFILE_LOCATION}"
        sudo chmod 600 "${SWAPFILE_LOCATION}"
        sudo mkswap "${SWAPFILE_LOCATION}"
        sudo swapon "${SWAPFILE_LOCATION}"
        echo "${SWAPFILE_LOCATION} none    swap    sw    0   0" | sudo tee -a /etc/fstab
    fi
}
function configure_touchpad {
    config_file="/usr/share/X11/xorg.conf.d/40-libinput.conf"
    # Tap to click
    grep --quiet 'Option "Tapping" "on"' "${config_file}" ||
        sudo sed 's/^\(\( *\)Identifier "[a-zA-Z0-9 ]*touchpad[a-zA-Z0-9 ]*" *\)$/\1\n\2Option "Tapping" "on"/' -i "${config_file}"

    # Fix touchpad after sleep
    echo '
#!/bin/sh

case $1 in
  post)
    /sbin/modprobe -r psmouse && /sbin/modprobe psmouse
  ;;
esac
' | sudo tee /lib/systemd/system-sleep/touchpad
}

function configure_tlp {
    # Enable charge threshold for tlp
    sudo sed -i 's/#\([A-Z]*_CHARGE_THRESH_BAT0=[0-9]*\)$/\1/' /etc/tlp.conf
    sudo systemctl restart tlp
}

function remove_snap_directories {
    sudo rm -rf /snap "$HOME"/snap
}

function main {
    set -xue

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
    remove_snap_directories
    set_global_keymap
    update_locales
}

if [ "$#" -ne 1 ] || [ "${1}" != "--source-only" ]; then
    main "${@}"
fi
