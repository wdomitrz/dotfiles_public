#!/usr/bin/env bash
source "${HOME}"/.local/bin/install_scripts/print_and_run.sh

function basic() {
  where="$1"

  print_and_run propagate_config_to.sh "${where}"
}

function sudo_write_to_remote_file() {
  where="$1"
  dest_path="$2"

  cat | ssh "${where}" 'dest="'"${dest_path}"'"; sudo mkdir -p "$(dirname "${dest}")" && sudo tee "${dest}" > /dev/null'
}

function copy_config_file() {
  where="$1"
  config_file_path="$2"

  # shellcheck disable=SC2002
  cat "${HOME}"/.config/"${config_file_path}" \
    | sudo_write_to_remote_file "${where}" "${config_file_path}"
}

function config_ssh() {
  where="$1"

  copy_config_file "${where}" /etc/ssh/sshd_config.d/01_my_sshd.conf
  ssh "${where}" 'sudo systemctl restart ssh.service'
}

function update_packages() {
  ssh "$1" 'sudo DEBIAN_FRONTEND=noninteractive apt-get update --yes --quiet=2'
}

function upgrade_packages() {
  ssh "$1" 'sudo DEBIAN_FRONTEND=noninteractive apt-get dist-upgrade --yes --quiet=2'
}

function install_packages() {
  where="$1"

  # shellcheck disable=SC2002
  cat "${HOME}"/.config/packages/packages_pi.sorted.txt \
    | ssh "${where}" 'xargs sudo DEBIAN_FRONTEND=noninteractive \
        apt-get install --yes --quiet=2'
}

function setup_unattended_upgrades() {
  where="$1"

  copy_config_file "${where}" /etc/apt/apt.conf.d/97_my_unattended_upgrades.conf
  ssh "${where}" 'sudo systemctl daemon-reload && sudo systemctl enable --now unattended-upgrades'
}

function setup_tailscale() {
  where="$1"

  ssh -t "${where}" 'curl -fsSL https://tailscale.com/install.sh | sh'
  ssh -t "${where}" 'sudo tailscale up'
}

function all() {
  where="$1"

  print_and_run basic "${where}"
  print_and_run config_ssh "${where}"
  print_and_run update_packages "${where}"
  print_and_run install_packages "${where}"
  print_and_run setup_unattended_upgrades "${where}"
  print_and_run setup_tailscale "${where}"
  print_and_run upgrade_packages "${where}"
}

function main_setup_pi() {
  set -euo pipefail
  where="$1"
  shift 1

  cmd="all"
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --run-only | --cmd) cmd="$2" && shift 2 ;;
      *) echo "Unknown param $1" && exit 1 ;;
    esac
  done

  print_and_run "${cmd}" "${where}"
}

if [[ $# -ne 1 ]] || [[ ${1} != "--source-only" ]]; then
  main_setup_pi "${@}"
fi
