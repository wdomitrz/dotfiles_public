#!/usr/bin/env sh
if [ "$(cat "${HOME}"/.config/git/dotfiles/main_branch.sorted.txt)" = \
    "$(git rev-parse --abbrev-ref HEAD)" ]; then
    dotfiles_git_merge_branches_post_commit.sh
fi
