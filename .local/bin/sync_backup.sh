#!/usr/bin/env bash

function copy_data() {
    set -euo pipefail

    from_location="$1"
    where_and_to_location="$2"

    echo "Copying data from ${from_location} to ${where_and_to_location}"
    rsync \
        --archive --verbose \
        --partial --progress \
        --exclude Oxford \
        "${from_location}/data/" "${where_and_to_location}/data"
}

function do_local_versioning() {
    set -euo pipefail

    to_location="$1"

    cd "${to_location}"
    set -x

    make
    git add -- files.txt files_only.txt
    git commit --message="sync up"

    set +x
    cd -
}

function push_up() {
    set -euo pipefail

    to_location="$1"

    cd "${to_location}"
    set -x

    git push
    git fetch

    set +x
    cd -
}

function main() {
    set -euo pipefail

    from_location="$1"
    where_and_to_location="$2"

    if [[ "${where_and_to_location}" != *:* ]]; then
        echo "Incorrect format of ${where_and_to_location}. Expected ':' inside."
        exit 1
    fi

    where="${where_and_to_location%%:*}"
    to_location="${where_and_to_location#*:}"

    if ! ssh "${where}" -- hostname; then
        echo "Cannot ssh to ${where}."
        exit 1
    fi

    copy_data "${from_location}" "${where}:${to_location}"
    ssh "${where}" -- "$(typeset -f); do_local_versioning" "${to_location}"
    ssh -A "${where}" -- "$(typeset -f); push_up" "${to_location}"
}

if [[ "$#" -ne 1 ]] || [[ "${1}" != "--source-only" ]]; then
    main "${@}"
fi
