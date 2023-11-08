#!/usr/bin/env sh

settings_file="${HOME}/.config/Code/User/settings.json"
if [ $# -ge 1 ]; then
    settings_file="$1"
fi

jq '."remote.SSH.defaultExtensions"[]' "${settings_file}" |
    xargs --max-lines=1 code --force --install-extension
