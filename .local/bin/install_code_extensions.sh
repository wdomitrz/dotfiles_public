#!/usr/bin/env bash

settings_file="${HOME}/.config/Code/User/settings.json"
if [[ $# -ge 1 ]]; then
    settings_file="$1"
fi

function list_extensions_to_install() {
    jq --raw-output '."remote.SSH.defaultExtensions"[]' "${settings_file}"
}

list_extensions_to_install |
    sed 's/^/--install-extension /g' | xargs --no-run-if-empty code --force

comm -23 \
    <(code --list-extensions | grep --invert-match "Extensions installed on " | sort) \
    <(list_extensions_to_install | sort) |
    sed 's/^/--uninstall-extension /g' | xargs --no-run-if-empty code
