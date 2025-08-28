#!/usr/bin/env sh
set -eu
sanitize_synced_files.sh
git diff --exit-code
