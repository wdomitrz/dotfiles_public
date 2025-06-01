#!/usr/bin/env bash
set +e
set -uo pipefail

kill_others_with_given_name.sh "$(basename "$0")"

created_files_dir="/dev/shm/i3status_files"
# shellcheck disable=SC2174
# --mode=700 applied only to the deepest directory - as intended
mkdir --parents --mode=700 "${created_files_dir}"

function cleanup_i3status_files_dir() {
    rm --recursive "${created_files_dir}"
}
trap cleanup_i3status_files_dir EXIT

file_updaters_dir="${HOME}"/.local/bin/i3status_file_updaters/
check_integrity_of_tracked_dir.sh "${file_updaters_dir}" || exit 1

readarray -t file_updaters < <(
    find "${file_updaters_dir}" -maxdepth 1 -not -name '.*' -type f -follow
)

for script_path in "${file_updaters[@]}"; do
    what="$(basename --suffix=".sh" "${script_path}")"
    while :; do
        output_file="${created_files_dir}"/"${what}".txt
        touch "${output_file}"
        chmod u=rw,g=,o= "${output_file}"
        "${script_path}" | sponge "${output_file}"
        sleep 10
    done &
done

wait
