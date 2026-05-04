#!/bin/sh

top="${1:-1}"
cursor_line="${2:-0}"
cursor_col="${3:-1}"

tmp="$(mktemp "${TMPDIR:-/tmp}/kitty-nvim-pager.XXXXXX")" || exit 1
cat > "${tmp}"
trap "rm -f '""${tmp}""'" EXIT

export KITTY_PAGER_FILE="${tmp}"
export KITTY_PAGER_TOP="${top}"
export KITTY_PAGER_CURSOR_LINE="${cursor_line}"
export KITTY_PAGER_CURSOR_COL="${cursor_col}"

exec nvim -n --clean \
  --cmd 'set eventignore=FileType' \
  -S "${HOME}/.config/kitty/pager.vim"
