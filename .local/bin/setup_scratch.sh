#!/usr/bin/env sh

SCRATCH_REAL_LOCATION=/dev/shm/scratch_"${USER}"
SCRATCH_LINK_LOCATION="${HOME}"/scratch

if [ -d "$(dirname "${SCRATCH_REAL_LOCATION}")" ]; then
    # shellcheck disable=SC2174
    mkdir --parents --mode=700 "${SCRATCH_REAL_LOCATION}" || true

    if [ ! -L "${SCRATCH_LINK_LOCATION}" ]; then ln --symbolic --force "${SCRATCH_REAL_LOCATION}" "${SCRATCH_LINK_LOCATION}"; fi
fi
