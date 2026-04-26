#!/usr/bin/env bash
function print_and_run() {
  printf '+'
  printf ' %q' "$@"
  printf '\n'
  "$@"
}

function download_and_verify {
  tmp_path="$(mktemp)"

  local -r checksum_sha256="$1"
  shift 1

  wget --no-verbose --no-show-progress --quiet --output-document=- "$@" > "${tmp_path}"

  if ! echo "${checksum_sha256} ${tmp_path}" | sha256sum --check --quiet; then
    echo "Wrong checksum for ${*}" 1>&2
    rm --force "${tmp_path}"
    return 1
  else
    cat "${tmp_path}"
    rm --force "${tmp_path}"
  fi
}
