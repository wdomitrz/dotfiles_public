#!/usr/bin/env bash
function format_json() { format_json.py; }
function format_py() { ruff format -; }
function format_sh() { shfmt --indent=2 --binary-next-line --case-indent --space-redirects --simplify -; }
function format_c_cpp() { clang-format -; }
function format_sorted_json() { LC_ALL=C format_json.py --sort-keys=True; }
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

  local -r in_file="$(mktemp)"
  local -r out_file="$(mktemp)"
  local -r err_file="$(mktemp)"
  # shellcheck disable=SC2064
  trap "rm --force -- '${in_file}' '${out_file}' '${err_file}'" EXIT

  set +e
  tee "${in_file}" | format_stdin "${filetype}" > "${out_file}" 2> "${err_file}"
  local -r formater_exit_code="${PIPESTATUS[1]}"
  set -e

  if [[ ${formater_exit_code} -ne 0 ]]; then
    cat "${in_file}"
    return "${formater_exit_code}"
  fi

  cat "${out_file}"
}

if [[ $# -ne 1 ]] || [[ ${1} != "--source-only" ]]; then
  format_stdin_main "${@}"
fi
