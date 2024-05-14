#!/usr/bin/env bash
file_updaters=()

function main() {
    for what in "${file_updaters[@]}"; do
        while :; do
            i3status_file_updaters_"${what}".sh | sponge "${HOME}"/.cache/i3status_files-"${what}".txt
            sleep 10
        done &
    done
}

if [[ "$#" -ne 1 ]] || [[ "${1}" != "--source-only" ]]; then
    main "${@}"
fi
