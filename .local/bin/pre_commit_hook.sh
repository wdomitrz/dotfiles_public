#!/usr/bin/env sh

sanitize_synced_files.sh
git diff --exit-code
