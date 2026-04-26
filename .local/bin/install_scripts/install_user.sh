#!/usr/bin/env bash
source "${HOME}"/.local/bin/install_scripts/utils.sh

function install_uv() {
  source "${HOME}"/.local/bin/install_scripts/install_global.sh --source-only

  archive_name='uv-x86_64-unknown-linux-gnu'
  install_to_given_location \
    "${HOME}"/.local/opt \
    "${HOME}"/.local/bin \
    not_sudo \
    "https://github.com/astral-sh/uv/releases/download/0.11.7/${archive_name}.tar.gz" \
    '6681d691eb7f9c00ac6a3af54252f7ab29ae72f0c8f95bdc7f9d1401c23ea868' \
    "${archive_name}"/uv \
    --ungzip

  ln --relative --symbolic --force \
    "${HOME}"/.local/opt/"${archive_name}"/uvx \
    "${HOME}"/.local/bin/
}

function install_ruff() {
  source "${HOME}"/.local/bin/install_scripts/install_global.sh --source-only

  archive_name='ruff-x86_64-unknown-linux-gnu'
  install_to_given_location \
    "${HOME}"/.local/opt \
    "${HOME}"/.local/bin \
    not_sudo \
    https://github.com/astral-sh/ruff/releases/download/0.15.12/"${archive_name}".tar.gz \
    '5e26a7811f7db364864ace00cced53003556a37b63f3c987e340b18207776f1c' \
    "${archive_name}"/ruff \
    --ungzip
}

function install_basedpyright() {
  uv tool install --quiet basedpyright
}

function install_doc() {
  local -r name="$1"
  shift 1
  local target_dir="${HOME}/.local/share/doc/${name}"
  local package_file
  package_file="$(mktemp /tmp/XXXXXX.zip)"

  mkdir --parents "${target_dir}"
  download_and_verify "$@" > "${package_file}"
  unzip -qud "${target_dir}"/ "${package_file}"
  rm "${package_file}"
}

function install_rmz_and_cpz() {
  subdir=fuc
  mkdir --parents "${HOME}"/.local/opt/"${subdir}"

  download_and_verify \
    '54f643c6ba170d613c65c48697000faf68d9c77611c10458ea5b1eac99799d25' \
    'https://github.com/SUPERCILEX/fuc/releases/download/3.0.1/x86_64-unknown-linux-gnu-rmz' \
    > "${HOME}"/.local/opt/"${subdir}"/rmz
  download_and_verify \
    'cf8147eda901948c643975e3c29d4b10db9fbfdc475585d57f1446dfaa2fa16f' \
    'https://github.com/SUPERCILEX/fuc/releases/download/3.0.1/x86_64-unknown-linux-gnu-cpz' \
    > "${HOME}"/.local/opt/"${subdir}"/cpz

  for what in rmz cpz; do
    chmod +x "${HOME}"/.local/opt/"${subdir}"/"${what}"
    ln --relative --symbolic --force \
      "${HOME}"/.local/opt/"${subdir}"/"${what}" \
      "${HOME}"/.local/bin/
  done
}

function install_nvim_plugins() {
  git submodule update --init --recursive
}

function install_user_main() {
  set -euo pipefail

  print_and_run install_uv
  print_and_run install_ruff
  print_and_run install_basedpyright
  print_and_run install_rmz_and_cpz
  print_and_run install_nvim_plugins
}

if [[ $# -ne 1 ]] || [[ ${1} != "--source-only" ]]; then
  install_user_main "${@}"
fi
