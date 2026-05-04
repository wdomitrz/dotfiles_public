#!/usr/bin/env sh
set -eu

top="${1:-1}"
cursor_line="${2:-0}"
cursor_col="${3:-1}"

tmp="$(mktemp "${TMPDIR:-/tmp}/kitty-nvim-pager.XXXXXX")" || exit 1
cat > "${tmp}"
trap "rm -f '""${tmp}""'" EXIT

exec env \
  KITTY_PAGER_FILE="${tmp}" \
  KITTY_PAGER_TOP="${top}" \
  KITTY_PAGER_CURSOR_LINE="${cursor_line}" \
  KITTY_PAGER_CURSOR_COL="${cursor_col}" \
  nvim -n --clean \
  -S "${HOME}/.config/kitty/pager.vim"
