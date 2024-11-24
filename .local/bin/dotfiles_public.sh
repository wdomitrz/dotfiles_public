#!/usr/bin/env bash

readonly public_upstream=public
readonly public_branch=main
readonly local_public_branch=origin/public
readonly public_repo=git@github.com:wdomitrz/dotfiles_public

function get_public() {
    git remote add "${public_upstream}" "${public_repo}"
    git fetch "${public_upstream}"
}

function checkout_public() {
    git checkout "${public_branch}"
}

function update_to_local_copy() {
    git checkout "${local_public_branch}" -- "$(git rev-parse --show-toplevel)"
    git diff "${local_public_branch}" --name-only --diff-filter=A |
        xargs --no-run-if-empty git rm
    git commit --message="some changes"
}

function compare_with_local_copy() {
    echo "DIFF BEGINS"
    git diff "${local_public_branch}"
    echo "DIFF ENDS"
}

function push() {
    git push
}

function remove_remote() {
    git remote remove "${public_upstream}"
}
function cleanup() {
    starting_branch="$1"
    git checkout "${starting_branch}"
    git branch --delete "${public_branch}"
    remove_remote
}

function dotfiles_publish_to_public_main() {
    set -euo pipefail
    starting_branch="$(git rev-parse --abbrev-ref HEAD)"
    get_public
    checkout_public
    update_to_local_copy
    push
    cleanup "${starting_branch}"
}

function diff_public() {
    get_public
    echo "DIFF BEGINS"
    git diff "${public_upstream}"/"${public_branch}" "${local_public_branch}"
    echo "DIFF ENDS"
    remove_remote
}

if [[ "$#" -ne 1 ]]; then
    echo "Expected exactly 1 argument"
    exit 1
fi

case "$1" in
diff)
    set -euo pipefail
    diff_public
    ;;
publish)
    set -xeuo pipefail
    dotfiles_publish_to_public_main
    ;;
--source-only)
    true
    ;;
*)
    echo "Unrecognized argument '$1'"
    exit 1
    ;;
esac
