#!/usr/bin/env bash
function format_json() { jq --compact-output . - | prettier --parser json; }
function format_py() { ruff format -; }
function format_sh() { shfmt --indent=2 --binary-next-line --case-indent --space-redirects --simplify -; }
function format_c_cpp() { clang-format -; }
function format_sorted_json() { LC_ALL=C jq --compact-output --sort-keys . - | format_json; }
function format_sorted_numeric_txt() { LC_ALL=C sort --numeric-sort -; }
function format_sorted_txt() { LC_ALL=C sort -; }
function format_vim() { format_nvim.sh vim; }
function format_lua() { format_nvim.sh lua; }

function format_stdin() {
  set -euo pipefail
  if [[ $# -ne 1 ]]; then
    echo "${FUNCNAME[*]}: Expected file type"
    return 1
  fi
  case "$1" in
    sh | bash | shellscript) format_sh ;;
    py | python) format_py ;;
    c | cpp | cuda) format_c_cpp ;;
    sorted_json) format_sorted_json ;;
    json | jsonc) format_json ;;
    vim) format_vim ;;
    lua) format_lua ;;
    sorted_txt | sorted) format_sorted_txt ;;
    sorted_numeric_txt | sorted_numeric) format_sorted_numeric_txt ;;
    unknown) cat ;;
    *)
      echo "Unsupported filetype '$1'"
      return 1
      ;;
  esac
}

function get_filetype() {
  case "$1" in
    *.sh | *.profile | *.bashrc | *.bash_aliases | *.bash_logout | \
      *.fehbg | *.Xclients | .xsessionrc | .xsession) echo sh ;;
    *.py) echo py ;;
    *.c | *.h) echo c ;;
    *.C | *.cc | *.cpp | *.H | *.hh | *.hpp) echo cpp ;;
    *.cu) echo cuda ;;
    *.sorted.json) echo sorted_json ;;
    *.json) echo json ;;
    *.vim) echo vim ;;
    *.lua) echo lua ;;
    *.sorted.txt) echo sorted_txt ;;
    *.sorted_numeric.txt) echo sorted_numeric_txt ;;
    *) echo unknown ;;
  esac
}

function format_stdin_main() {
  if [[ $# -eq 0 ]]; then
    echo "${FUNCNAME[*]}: Expected a flag with value"
    return 1
  fi
  case "$1" in
    --filename)
      filetype="$(get_filetype "$2")"
      shift 2
      ;;
    --filetype)
      filetype="$2"
      shift 2
      ;;
    *) echo "Unsupported argument: $1" && return 1 ;;
  esac

  format_stdin "${filetype}"
}

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

  # Only supported file types
  filetype="$(get_filetype "$1")"

  if [[ ${filetype} == "unknown" ]]; then
    return 0
  fi

  # shellcheck disable=SC2002
  cat "${given_file_path}" \
    | format_stdin "${filetype}" \
    | sponge "${given_file_path}"
}

function format_files_main() {
  if [[ $# -eq 0 ]]; then
    echo "${FUNCNAME[*]}: Expected files paths"
    return 1
  fi
  while IFS= read -r -d '' f; do
    format_file "${f}"
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
      LC_ALL=C format_stdin_main "$@"
      ;;
    files)
      shift 1
      LC_ALL=C format_files_main "$@"
      ;;
    *) echo "Unsupported subcommand: $1" && return 1 ;;
  esac
}

if [[ $# -ne 1 ]] || [[ ${1} != "--source-only" ]]; then
  LC_ALL=C format_main "${@}"
fi
