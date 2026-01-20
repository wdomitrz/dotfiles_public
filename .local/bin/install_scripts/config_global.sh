#!/usr/bin/env bash
source "${HOME}"/.local/bin/install_scripts/print_and_run.sh

function copy_global_configs() {
  check_integrity_of_tracked_dir.sh "${HOME}"/.config/etc \
    && sudo cp --update --backup=numbered --recursive \
      "${HOME}"/.config/etc/. /etc/
}

function regenerate_grub_post_config_copy() {
  sudo update-initramfs -u > /dev/null
  sudo update-grub 2> /dev/null
}

function reconfigure_tlp_post_config_copy() {
  sudo systemctl restart tlp || true
}

function add_user_to_groups() {
  for group in sudo audio video users netdev bluetooth docker lpadmin nordvpn kvm input uinput libvirt; do
    sudo groupadd --force "${group}"
    sudo usermod --append --groups "${group}" "${USER}" \
      || echo "Adding to ${group} failed"
  done
}

function enable_32_bit_architecture() {
  sudo dpkg --add-architecture i386
}

function update_locales() {
  sudo dpkg-reconfigure --frontend noninteractive locales 2> /dev/null
}

function create_swap_file() {
  local -r swapfile_location="/home/swapfile"
  if ! [[ -f ${swapfile_location} ]]; then
    sudo fallocate -l 2G "${swapfile_location}"
    sudo chmod 600 "${swapfile_location}"
    sudo mkswap "${swapfile_location}"
    sudo swapon "${swapfile_location}"
    echo "${swapfile_location} none    swap    sw    0   0" \
      | sudo tee -a /etc/fstab
  fi
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

function todo_post_global_configs_copy() {
  print_and_run regenerate_grub_post_config_copy
  print_and_run reconfigure_tlp_post_config_copy
}

function config_global_start() {
  print_and_run copy_global_configs
  print_and_run update_locales
  print_and_run enable_32_bit_architecture
}

function config_global_rest() {
  print_and_run todo_post_global_configs_copy
  print_and_run add_user_to_groups
  print_and_run udisk_allow_operations
  print_and_run create_swap_file
}

function config_global_main() {
  set -euo pipefail

  # Only the part that is not used in install_packages
  print_and_run config_global_rest
}

if [[ $# -ne 1 ]] || [[ ${1} != "--source-only" ]]; then
  config_global_main "${@}"
fi
