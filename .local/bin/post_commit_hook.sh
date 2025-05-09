#!/usr/bin/env sh
main_branch_file="${HOME}"/.config/git/dotfiles/main_branch.sorted.txt

if [ -e "${main_branch_file}" ] &&
    [ "$(cat "${main_branch_file}")" = "$(git rev-parse --abbrev-ref HEAD)" ]; then
    dotfiles_git_merge_branches_post_commit.sh
fi
