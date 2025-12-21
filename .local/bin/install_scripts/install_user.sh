#!/usr/bin/env bash

function install_haskell_ghcup() {
  curl --fail --progress-bar --show-error --location --proto '=https' --tlsv1.2 https://get-ghcup.haskell.org | sh
}

function install_nvim() {
  source "${HOME}"/.local/bin/install_scripts/install_global.sh --source-only
  install_nvim_given_locations "${HOME}"/.local/opt "${HOME}"/.local/bin not_sudo
}

function install_uv() {
  wget_with_defaults.sh --max-redirect=3 https://astral.sh/uv/install.sh | sh
}

function install_python_tools() {
  install_uv
  uv tool install basedpyright --force
  uv tool install ruff --force
}

function install_doc() {
  local url="$1"
  local name="$2"
  local target_dir="${HOME}/.local/share/doc/${name}"
  local package_file
  package_file="$(mktemp /tmp/XXXXXX.zip)"

  mkdir --parents "${target_dir}"
  wget_with_defaults.sh "${url}" > "${package_file}"
  unzip -qud "${target_dir}"/ "${package_file}"
  rm "${package_file}"
}

function get_binary_from() {
  name="$1"
  checksum_sha256="$2"
  url="$3"
  shift 3

  file_path="${HOME}/.local/bin/${name}"
  wget_with_defaults.sh "$@" "${url}" > "${file_path}"
  if ! echo "${checksum_sha256} ${file_path}" | sha256sum --check; then
    echo "Wrong checksum for ${url}"
    return 1
  fi

  chmod +x "${file_path}"
}

function install_rmz_and_cpz() {
  get_binary_from rmz \
    "54f643c6ba170d613c65c48697000faf68d9c77611c10458ea5b1eac99799d25" \
    "https://github.com/SUPERCILEX/fuc/releases/download/3.0.1/x86_64-unknown-linux-gnu-rmz" \
    --max-redirect=1
  get_binary_from cpz \
    "cf8147eda901948c643975e3c29d4b10db9fbfdc475585d57f1446dfaa2fa16f" \
    "https://github.com/SUPERCILEX/fuc/releases/download/3.0.1/x86_64-unknown-linux-gnu-cpz" \
    --max-redirect=1
}

function install_nvim_plugins() {
  git submodule update --init --recursive
}

function install_user_main() {
  set -euo pipefail
  set -x

  install_python_tools
  install_rmz_and_cpz
  install_nvim_plugins
}

if [[ $# -ne 1 ]] || [[ ${1} != "--source-only" ]]; then
  install_user_main "${@}"
fi
