#!/usr/bin/env sh
rsync \
    --archive --verbose \
    --partial --progress \
    --exclude Oxford \
    "$@"
