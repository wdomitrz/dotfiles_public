#!/usr/bin/env bash
set +e
set -uo pipefail

file_updaters_dir="${HOME}"/.local/bin/i3status_file_updaters/
check_integrity_of_tracked_dir "${file_updaters_dir}" || exit 1

readarray -t file_updaters < <(
    find "${file_updaters_dir}" -maxdepth 1 -not -name '.*' -type f -follow
)

for script_path in "${file_updaters[@]}"; do
    what="$(basename --suffix=".sh" "${script_path}")"
    while :; do
        "${script_path}" | sponge "${HOME}"/.cache/i3status_files-"${what}".txt
        sleep 10
    done &
done
