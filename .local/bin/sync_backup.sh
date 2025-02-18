#!/usr/bin/env bash
set -euo pipefail

from_location="$1/data/"
to_location="$2/data/"

rsync \
    --archive --verbose \
    --partial --progress \
    --exclude Oxford \
    "${from_location}" "${to_location}"
