#!/usr/bin/env bash
function format_all_files() {
  git-ls | grep -v '.local/state/nvim/site/pack/plugins/opt/' | xargs format.sh files
}

function lint_sources_files() {
  for f in $(git-ls | grep "\.sourcesk$"); do
    if grep --quiet '^Enabled: no$' "${f}" \
      && grep --quiet '^Signed-By:' "${f}"; then
      echo "${f} is not enabled, but already signed" && return 1
    fi
    if (! grep --quiet '^Signed-By:' "${f}") \
      && (! grep --quiet '^key_source:' "${f}"); then
      echo "${f} is not signed, and doesn't have a key_source" && return 1
    fi
    if grep --quiet '^Enabled: yes$' "${f}" \
      && grep --quiet '^Signed-By:' "${f}"; then
      echo "${f} is enabled, but not signed" && return 1
    fi
  done
}

function lint_shell_files() {
  git_root="$(git rev-parse --show-toplevel)"
  (
    # Automatically apply fixes
    git-ls | grep "\.sh$" | xargs readlink -f \
      | parallel shellcheck --exclude=SC1091,SC2312 --enable=all --format=diff \
      | sed "s|--- ${git_root}|--- a|g" \
      | sed "s|+++ ${git_root}|+++ b|g" \
      | patch --strip=1 --directory="${git_root}"
  ) || (
    # Show all errors
    git-ls | grep "\.sh$" \
      | parallel shellcheck --exclude=SC1091,SC2312 --enable=all
  )
}

function lint_python_files() {
  git-ls | grep "\.py$" | xargs readlink -f \
    | xargs ruff check --quiet --extend-select I --fix
}

function type_python_files() {
  git-ls | grep "\.py$" | xargs readlink -f \
    | xargs basedpyright --pythonversion 3.10 --project "${HOME}"/.config/python/pyproject.toml \
    | not grep --invert-match "0 errors, 0 warnings, 0 notes"

}

function get_all_files_without_extensions() {
  for file in $(git-ls | grep -v '.local/state/nvim/site/pack/plugins/opt/'); do
    [[ -L ${file} ]] && continue # Ignore links
    base_file_name="$(basename "${file}")"
    [[ ${base_file_name} == ?*.* ]] && continue    # Check if the file has an extension
    [[ ${base_file_name} == ".keep" ]] && continue # Check if the file is a keep file
    echo "${file}"
  done
}

function get_all_files_covered_by_extension_links() {
  git-ls \
    | (grep ".config/extension_links/" || true) \
    | xargs --no-run-if-empty readlink -f \
    | xargs --no-run-if-empty realpath -q \
    | sed "s|^$(pwd)/||g"
}

function dangling_extension_links() {
  for file in $(git-ls \
    | grep ".config/extension_links/"); do
    [[ -L ${file} ]] || continue # Process only links
    readlink -f "${file}" > /dev/null \
      || echo "${file}"
  done
}

function lint_extension_links() {
  dangling_extension_links | not grep "?*"

  get_all_files_covered_by_extension_links \
    | (grep "^.config/extension_links/" || true) \
    | not grep "?*"

  comm -23 <(
    get_all_files_without_extensions | sort
  ) <(
    get_all_files_covered_by_extension_links | sort
  ) | not grep "?*"
}

# Parallel execution with tracking processes ###################################
function run_and_save() {
  set -euo pipefail
  name="$1"
  (
    "$@"
    echo -n "${name}"
    echo -e "\tdone"
  ) &
}

function gitignore_regenerate() {
  git_root="$(git rev-parse --show-toplevel)"
  (
    echo '*'
    git-ls | sed 's/^/!/g' | LC_ALL=C sort
  ) > "${git_root}"/.gitignore
}

function sanitize_synced_main() {
  source "${HOME}"/.profile

  run_and_save gitignore_regenerate
  run_and_save lint_sources_files
  run_and_save lint_python_files
  run_and_save lint_extension_links
  run_and_save lint_shell_files
  run_and_save format_all_files
  run_and_save type_python_files
  wait
}

################################################################################

if [[ $# -ne 1 ]] || [[ ${1} != "--source-only" ]]; then
  set -euo pipefail

  LC_ALL=C sanitize_synced_main "${@}"
fi
