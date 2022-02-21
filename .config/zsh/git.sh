if [ -f /usr/lib/git-core/git-sh-prompt ]; then
    # Use git_ps1 to show git info
    export GIT_PS1_ENABLED=true

    source /usr/lib/git-core/git-sh-prompt

    export GIT_PS1_SHOWDIRTYSTATE=true     # staged '+', unstaged '*'
    export GIT_PS1_SHOWSTASHSTATE=true     # '$' something is stashed
    export GIT_PS1_SHOWUNTRACKEDFILES=true # '%' untracked files
    export GIT_PS1_SHOWUPSTREAM="auto"     # '<' behind, '>' ahead, '<>' diverged, '=' no difference
    export GIT_PS1_UNTRACKEDFILES=true
    export GIT_PS1_SHOWCOLORHINTS=true

    export PROMPT_COMMAND='__git_ps1 "${PROMPT_FRONT}" "${PROMPT_BACK}"'
fi

setopt prompt_subst
autoload -Uz vcs_info
zstyle ':vcs_info:*' enable hg svn
# If git handled by git-propmt.sh, then don't double it
[ -z "GIT_PS1_ENABLED" ] && zstyle ':vcs_info:*' enable git
zstyle ':vcs_info:*' check-for-changes true
zstyle ':vcs_info:*' stagedstr '%F{green}+%f'
zstyle ':vcs_info:*' unstagedstr '%F{red}*%f'
zstyle ':vcs_info:*' formats " (%F{green}%b%u%c%m%f)"
zstyle ':vcs_info:*' actionformats " (%F{green}%b%m%F{yello}}|%f{red}%a%u%c%f)"

precmd() {
    vcs_info
    if [ ! -z "GIT_PS1_ENABLED" ]; then
        __git_ps1 "${PROMPT_FRONT}" "${vcs_info_msg_0_}${PROMPT_BACK}"
    else
        PROMPT="${PROMPT_FRONT}${vcs_info_msg_0_}${PROMPT_BACK}"
    fi
}

source /usr/lib/git-core/git-sh-prompt
