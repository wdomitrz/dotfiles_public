#!/usr/bin/env bash
set -ue

function get_config_file() {
  readonly original_file_path="$1"
  # shellcheck disable=SC2155
  readonly tmp_file_path=/tmp/"$(basename "${original_file_path}")"
  [[ -f ${original_file_path} ]] && (cp -a "${original_file_path}" "${tmp_file_path}" || true)
  echo "${tmp_file_path}"
}

MAIN_BRANCH="$(set -e && get_config_file "${HOME}"/.config/git/dotfiles/main_branch.sorted.txt | xargs cat)"
readarray -t ALL_BRANCHES < <(get_config_file "${HOME}"/.config/git/dotfiles/all_branches.sorted.txt | xargs cat)

function merge_with_main_branch {
  current_branch="$1"
  git checkout "${current_branch}"
  if ! git merge --no-edit "${MAIN_BRANCH}"; then
    git status -s | grep "^DU " |
      cut --delimiter ' ' --fields 2- |
      xargs git rm
    git -c core.editor=true merge --continue
  fi
}

function go_back_push_and_cleanup {
  git checkout "${MAIN_BRANCH}"
  git push --all --force-with-lease
  git branch --delete "${MAIN_BRANCH}" "${ALL_BRANCHES[@]}" || echo "Branches removed"
}

if [[ ${MAIN_BRANCH} != "$(git rev-parse --abbrev-ref HEAD)" ]]; then
  git -c core.editor=true merge --continue
fi

git fetch

for branch in "${ALL_BRANCHES[@]}"; do
  merge_with_main_branch "${branch}"
done

go_back_push_and_cleanup
