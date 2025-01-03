#!/usr/bin/env sh
rsync \
    --archive --verbose \
    --partial --progress \
    --exclude Oxford \
    --exclude '*.sha256' \
    --exclude .git --exclude .gitignore --exclude Makefile \
    --exclude files.txt --exclude files_only.txt \
    "$@"
