#!/usr/bin/env sh

settings_file="${HOME}/.config/Code/User/settings.json"
if [ $# -ge 1 ]; then
  settings_file="$1"
fi

code --list-extensions | grep --invert-match "^Extensions installed on " \
  | jq --null-input --raw-input '{"remote.SSH.defaultExtensions": [inputs]}' \
  | jq 'add' --slurp "${settings_file}" - \
  | format.sh stdin --filetype sorted_json \
  | sponge "${settings_file}"
