#!/usr/bin/env bash
set -euo pipefail

settings_file="${HOME}/.config/Code/User/settings.json"
if [[ $# -ge 1 ]]; then
  settings_file="$1"
fi

function list_extensions_to_install() {
  jq --raw-output '."remote.SSH.defaultExtensions"[]?' "${settings_file}"
}

function install_extensions() {
  list_extensions_to_install \
    | sed 's/^/--install-extension /g' | xargs --no-run-if-empty code --force
}

install_output="$(install_extensions)"
if [[ -n ${install_output} ]]; then
  printf '%s\n' "${install_output}" \
    | grep --invert-match --fixed-strings 'Installing extensions...' \
    | grep --invert-match ' is already installed.$' \
    || true
fi

comm -23 \
  <(code --list-extensions | grep --invert-match "Extensions installed on " | sort) \
  <(list_extensions_to_install | sort) \
  | sed 's/^/--uninstall-extension /g' | xargs --no-run-if-empty code
