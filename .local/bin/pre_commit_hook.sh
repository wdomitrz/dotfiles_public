#!/usr/bin/env sh

sanitize_synced_files
git diff --exit-code
