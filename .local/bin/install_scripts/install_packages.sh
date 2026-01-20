#!/usr/bin/env bash
source "${HOME}"/.local/bin/install_scripts/print_and_run.sh

function update_and_upgrade() {
  sudo DEBIAN_FRONTEND=noninteractive apt-get update --yes --quiet=2 \
    && sudo DEBIAN_FRONTEND=noninteractive apt-get dist-upgrade --yes --quiet=2
}

function install_packages_from() {
  packages_file="$1"
  xargs sudo DEBIAN_FRONTEND=noninteractive \
    apt-get install --yes --quiet=2 < "${packages_file}"
}

function install_packages_from_main() {
  install_packages_from "${HOME}"/.config/packages/packages_main.sorted.txt
}

function install_packages_from_other() {
  install_packages_from "${HOME}"/.config/packages/packages_other.sorted.txt
}

function install_packages_from_nvidia() {
  if ! (lspci | grep --quiet --ignore-case nvidia); then
    return 0
  else
    install_packages_from "${HOME}"/.config/packages/packages_nvidia.sorted.txt
  fi
}

function install_packages_main() {
  set -euo pipefail
  source "${HOME}"/.local/bin/install_scripts/config_global.sh --source-only

  print_and_run copy_global_configs
  print_and_run update_and_upgrade
  print_and_run install_packages_from_main
  print_and_run config_global_start
  print_and_run update_and_upgrade
  print_and_run install_packages_from_other
  print_and_run install_packages_from_nvidia
}

if [[ $# -ne 1 ]] || [[ ${1} != "--source-only" ]]; then
  install_packages_main "${@}"
fi
