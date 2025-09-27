#!/usr/bin/env sh
set -eu
file_extension="$1"
tmp_file="$(mktemp --suffix=."${file_extension}")"
cat > "${tmp_file}"

nvim --clean --headless \
  -c 'set tabstop=4 softtabstop=-1 shiftwidth=0 expandtab' \
  -c 'silent norm gg=G' \
  -c 'silent wqa' \
  -- "${tmp_file}" \
  2> /dev/null

cat "${tmp_file}"
rm -f "${tmp_file}"
