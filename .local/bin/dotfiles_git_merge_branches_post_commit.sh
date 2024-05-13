#!/usr/bin/env bash
set -ue

MAIN_BRANCH="$(cat "${HOME}"/.config/git/dotfiles/main_branch.sorted.txt)"
readarray -t ALL_BRANCHES <"${HOME}"/.config/git/dotfiles/all_branches.sorted.txt

function merge_with_main_branch {
    current_branch="$1"
    git checkout "${current_branch}"
    git merge "${MAIN_BRANCH}"
}

function go_back_push_and_cleanup {
    git checkout "${MAIN_BRANCH}"
    git push --all --force-with-lease
    git branch -d "${MAIN_BRANCH}" "${ALL_BRANCHES[@]}" || echo "Branches removed"
}

if [[ "${MAIN_BRANCH}" != "$(git rev-parse --abbrev-ref HEAD)" ]]; then
    git -c core.editor=true merge --continue
fi

for branch in "${ALL_BRANCHES[@]}"; do
    merge_with_main_branch "${branch}"
done

go_back_push_and_cleanup
