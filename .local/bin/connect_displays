#!/usr/bin/env bash

# file with groups of comma separated displays
config_file="${HOME}/.config/displays_to_connect/${HOSTNAME}.txt"

[[ -f "${config_file}" ]] || exit 0

while read -r displays_to_connect; do
    xrandr --setmonitor "connected_${displays_to_connect}" auto "${displays_to_connect}"
done < "${config_file}"
