#!/usr/bin/env sh

simple_lsp_server.py \
  --format-command 'format.sh stdin --filetype sh' \
  --diagnostics-command 'shellcheck - --exclude=SC1091,SC2312 --enable=all --format=json1' \
  --code-actions-command 'shellcheck - --exclude=SC1091,SC2312 --enable=all --format=json1' \
  "$@"
