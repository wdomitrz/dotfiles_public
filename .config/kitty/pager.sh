#!/usr/bin/env sh
set -eu

. "${HOME}"/.profile

top="${1:-1}"
cursor_line="${2:-0}"
cursor_col="${3:-1}"

tmp="$(mktemp)"
# shellcheck disable=SC2064
trap "rm -f '${tmp}'" EXIT

cat > "${tmp}"

if command -v nvim > /dev/null 2>&1; then
  exec nvim -u NONE \
    -c "let g:kitty_scrollback_file = '${tmp}'" \
    -c "let g:kitty_scrollback_top = ${top}" \
    -c "let g:kitty_scrollback_line = ${cursor_line}" \
    -c "let g:kitty_scrollback_col = ${cursor_col}" \
    -S "${HOME}/.config/kitty/pager.vim"
else
  exec less -R "+${top}g" "${tmp}"
fi
