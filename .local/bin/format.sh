#!/usr/bin/env bash
function format_file() {
  set -euo pipefail
  if [[ $# -eq 0 ]]; then
    echo "${FUNCNAME[*]}: Expected file path"
    return 1
  fi
  given_file_path="$1"

  resolved_file_path="$(realpath --quiet "${given_file_path}")" \
    || return 0 # Ignore dangling links
  if [[ -d ${resolved_file_path} ]] || [[ ! -w ${resolved_file_path} ]]; then
    return 0 # Ignore directories and non-writeable files
  fi

  # shellcheck disable=SC2002
  cat "${given_file_path}" \
    | format_stdin.sh --filename "${given_file_path}" \
    | sponge "${given_file_path}"
}

function format_files_main() {
  if [[ $# -eq 0 ]]; then
    echo "${FUNCNAME[*]}: Expected files paths"
    return 1
  fi
  while IFS= read -r -d '' f; do
    format_file "${f}" &
  done < <(find "$@" -print0)
  wait
}

function format_main() {
  set -euo pipefail
  if [[ $# -eq 0 ]]; then
    echo "${FUNCNAME[*]}: Expected subcommand"
    return 1
  fi
  case "$1" in
    stdin)
      shift 1
      format_stdin.sh "$@"
      ;;
    files)
      shift 1
      format_files_main "$@"
      ;;
    *) echo "Unsupported subcommand: $1" && return 1 ;;
  esac
}

if [[ $# -ne 1 ]] || [[ ${1} != "--source-only" ]]; then
  format_main "${@}"
fi
