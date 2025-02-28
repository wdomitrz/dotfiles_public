#!/usr/bin/env bash

function sort_json_files() {
    list_git_files.sh | grep "\.sorted\.json$" |
        xargs format_sorted_json.sh
}

function format_json_files() {
    list_git_files.sh | grep "\.json$" |
        xargs format_json.sh
}

function do_all_jsons() {
    sort_json_files
    format_json_files
}

function sort_txt_files() {
    list_git_files.sh | grep "\.sorted\.txt$" |
        xargs format_sorted_txt.sh
}

function sort_numeric_txt_files() {
    list_git_files.sh | grep "\.sorted_numeric\.txt$" |
        xargs format_sorted_numeric_txt.sh
}

function format_vim_files() {
    list_git_files.sh | grep "\.vim$" |
        xargs format_vim.sh
}

function format_shell_files() {
    list_git_files.sh | grep "\.sh$" | xargs readlink --canonicalize |
        xargs format_sh.sh
}

function validate_sources_files() {
    for f in $(list_git_files.sh | grep "\.sourcesk$"); do
        if grep --quiet '^Enabled: no$' "${f}" &&
            grep --quiet '^Signed-By:' "${f}"; then
            echo "${f} is not enabled, but already signed" && return 1
        fi
        if (! grep --quiet '^Signed-By:' "${f}") &&
            (! grep --quiet '^key_source:' "${f}"); then
            echo "${f} is not signed, and doesn't have a key_source" && return 1
        fi
        if grep --quiet '^Enabled: yes$' "${f}" &&
            grep --quiet '^Signed-By:' "${f}"; then
            echo "${f} is enabled, but not signed" && return 1
        fi
    done
}

function apply_suggestions_to_shell_files() {
    git_root="$(git rev-parse --show-toplevel)"
    list_git_files.sh | grep "\.sh$" | xargs readlink --canonicalize |
        xargs shellcheck --exclude=SC1091,SC2312 --enable=all --format=diff |
        sed "s|--- ${git_root}|--- a|g" |
        sed "s|+++ ${git_root}|+++ b|g" |
        patch --strip=1 --directory="${git_root}"
}

function lint_shell_files() {
    list_git_files.sh | grep "\.sh$" |
        xargs shellcheck --exclude=SC1091,SC2312 --enable=all
}

function do_all_shell_files() {
    format_shell_files
    apply_suggestions_to_shell_files
    lint_shell_files
}

function format_python_files() {
    list_git_files.sh | grep "\.py$" | xargs readlink --canonicalize |
        xargs format_py.sh
}

function lint_python_files() {
    list_git_files.sh | grep "\.py$" | xargs readlink --canonicalize |
        xargs ruff check --quiet --extend-select I --fix
}

function do_all_python_files() {
    format_python_files
    lint_python_files
}

function get_all_files_without_extensions() {
    for file in $(list_git_files.sh); do
        [[ -L "${file}" ]] && continue # Ignore links
        base_file_name="$(basename "${file}")"
        [[ "${base_file_name}" == ?*.* ]] && continue    # Check if the file has an extension
        [[ "${base_file_name}" == ".keep" ]] && continue # Check if the file is a keep file
        echo "${file}"
    done
}

function get_all_files_covered_by_extension_links() {
    git_root="$(git rev-parse --show-toplevel)"
    list_git_files.sh |
        (grep "^.config/extension_links/" || true) |
        sed "s|^|${git_root}/|g" |
        xargs --no-run-if-empty readlink --canonicalize |
        sed "s|^${git_root}/||g"
}

function dangling_extension_links() {
    git_root="$(git rev-parse --show-toplevel)"
    for file in $(list_git_files.sh |
        grep "^.config/extension_links/" |
        sed "s|^|${git_root}/|g"); do
        [[ -L "${file}" ]] || continue # Process only links
        readlink --canonicalize-existing "${file}" > /dev/null ||
            echo "${file}"
    done
}

function files_without_extensions() {
    dangling_extension_links | not grep "?*"

    get_all_files_covered_by_extension_links |
        (grep "^.config/extension_links/" || true) |
        not grep "?*"

    comm -23 <(
        get_all_files_without_extensions | sort
    ) <(
        get_all_files_covered_by_extension_links | sort
    ) | not grep "?*"
}

# Parallel execution with tracking processes ###################################
declare -A running_processes
declare -a running_processes_names

function run_and_save() {
    set -euo pipefail
    name="$1"
    running_processes_names+=("${name}")
    "$@" &
    running_processes["${name}"]="$!"
}

function wait_for_all() {
    for name in "${running_processes_names[@]}"; do
        pid="${running_processes[${name}]}"
        echo -n "${name}"
        wait "${pid}"
        echo -e "\tdone"
    done
}

function sanitize_synced_main() {
    run_and_save do_all_jsons
    run_and_save files_without_extensions
    run_and_save sort_txt_files
    run_and_save sort_numeric_txt_files
    run_and_save validate_sources_files
    run_and_save format_vim_files
    run_and_save do_all_python_files
    run_and_save do_all_shell_files
    wait_for_all
}

################################################################################

if [[ "$#" -ne 1 ]] || [[ "${1}" != "--source-only" ]]; then
    set -euo pipefail

    sanitize_synced_main "${@}"
fi
