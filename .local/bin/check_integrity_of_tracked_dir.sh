#!/usr/bin/env bash

function check_if_all_files_are_tracked() {
    set -uo pipefail
    local files_location="$1"
    diff <(
        find "${files_location}" -type f -follow |
            sed --regexp-extended "s|^${HOME}/||" | sort
    ) <(
        git --git-dir="${HOME}"/.git ls-tree --full-tree --name-only -r HEAD "${files_location}" |
            sort
    )
}

function check_integrity_of_tracked_files() {
    set -uo pipefail
    local files_location="$1"
    git --git-dir="${HOME}"/.git diff --exit-code HEAD "${files_location}"
}

function checko_integrity_of_tracked_dir() {
    set -uo pipefail
    local files_location="$1"
    check_if_all_files_are_tracked "${files_location}" &&
        check_integrity_of_tracked_files "${files_location}"
}

if [[ $# -ne 1 ]] || [[ ${1} != "--source-only" ]]; then
    checko_integrity_of_tracked_dir "${@}"
fi
