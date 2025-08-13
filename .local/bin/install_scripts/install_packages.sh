#!/usr/bin/env bash

function update_and_upgrade() {
  sudo apt-get update --yes &&
    sudo apt-get dist-upgrade --yes
}

function install_packages_from() {
  packages_file="$1"
  xargs sudo DEBIAN_FRONTEND=noninteractive apt-get install --yes --no-install-recommends < "${packages_file}"
}

function install_packages_from_main() {
  install_packages_from "${HOME}"/.config/packages/packages_main.sorted.txt
}

function install_packages_from_other() {
  install_packages_from "${HOME}"/.config/packages/packages_other.sorted.txt
}

function install_packages_from_nvidia() {
  if ! (lspci | grep --quiet --ignore-case nvidia); then
    echo "nvidia driver not needed"
    return 0
  fi
  install_packages_from "${HOME}"/.config/packages/packages_nvidia.sorted.txt
}

function install_packages_main() {
  set -euo pipefail
  set -x
  source "${HOME}"/.local/bin/install_scripts/config_global.sh --source-only

  copy_global_configs
  update_and_upgrade
  install_packages_from_main
  config_global_start
  update_and_upgrade
  install_packages_from_other
  install_packages_from_nvidia
}

if [[ $# -ne 1 ]] || [[ ${1} != "--source-only" ]]; then
  install_packages_main "${@}"
fi
