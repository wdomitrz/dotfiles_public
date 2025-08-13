#!/usr/bin/env sh
tmp_file="$(mktemp --suffix=.vim)"
cat > "${tmp_file}"

nvim --clean --headless \
  -c 'set tabstop=4 softtabstop=-1 shiftwidth=0 expandtab' \
  -c 'silent norm gg=G' \
  -c 'silent wqa' \
  -- "${tmp_file}" \
  2> /dev/null

cat "${tmp_file}"
rm --force "${tmp_file}"
