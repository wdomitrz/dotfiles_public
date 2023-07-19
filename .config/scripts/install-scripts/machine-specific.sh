#!/usr/bin/env bash

function main() {
    set -xue
}

if [ "$#" -ne 1 ] || [ "${1}" != "--source-only" ]; then
    main "${@}"
fi
