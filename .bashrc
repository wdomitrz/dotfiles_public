#!/usr/bin/env bash
if [[ -n ${BASHRC_LOADED-} ]]; then return; fi
BASHRC_LOADED=1

# Profile
if [[ -f "${HOME}/.profile" ]]; then source "${HOME}/.profile"; fi

# If not running interactively, don't do anything
if [[ $- != *i* ]]; then return; fi

if [[ -f "/etc/bashrc" ]]; then source /etc/bashrc; fi

if [[ -f "${HOME}/.config/local/bashrc.sh" ]]; then source "${HOME}/.config/local/bashrc.sh"; fi

# Options
shopt -s autocd       # Automatically change directory
shopt -s checkwinsize # Detect resize
shopt -s extglob      # Use additional pattern matching features.
shopt -s globstar     # Enable **
shopt -s histappend   # Append to the history file
shopt -u failglob     # Unmatched patterns don't cause errors

set -o physical # symbolic links to directories expend directly to the target

HISTFILE="${HOME}/.cache/bash_history" # History file

HISTCONTROL=ignoredups # don't save lines with leading space
HISTSIZE=              # unlimited bash history
HISTFILESIZE=          # unlimited bash history

# Key bindings
## vi keybindings
set -o vi
## Search backward
bind '"\x08":backward-kill-word'
## Tab completion
bind 'TAB':menu-complete
# Perform partial completion on the first Tab
bind "set menu-complete-display-prefix on"

yank_line_to_clipboard() {
    echo "${READLINE_LINE}" | xclip -in -selection clipboard
}
bind -m vi-command -x '"yy": yank_line_to_clipboard'

kill_line_to_clipboard() {
    yank_line_to_clipboard
    READLINE_LINE=""
}
bind -m vi-command -x '"dd": kill_line_to_clipboard'

# enable programmable completion features
if ! shopt -oq posix; then
    if [[ -f /usr/share/bash-completion/bash_completion ]]; then
        source /usr/share/bash-completion/bash_completion
    elif [[ -f /etc/bash_completion ]]; then
        source /etc/bash_completion
    fi
fi

# Prompt
## Detect no color support
([[ -x /usr/bin/tput ]] && tput setaf 1 >&/dev/null) || NO_COLORS=1

if [[ -z ${NO_COLORS-} ]]; then
    # Trick to make colors work well in tmux and in other scenarios
    [[ ${TERM} != "xterm-kitty" ]] && TERM=xterm-256color

    # Colors
    USER_COLOR="\[$(tput setaf 10)\]"       # green
    AT_COLOR="\[$(tput setaf 11)\]"         # yellow
    HOST_COLOR="\[$(tput setaf 14)\]"       # cyan
    EXIT_STATUS_COLOR="\[$(tput setaf 9)\]" # red
    DATE_COLOR="\[$(tput setaf 208)\]"      # orange
    TIME_COLOR="\[$(tput setaf 13)\]"       # magenta
    PATH_COLOR="\[$(tput setaf 6)\]"        # light blue
    PROMPT_COLOR="\[$(tput setaf 4)\]"      # blue
    CLEAR="\[$(tput sgr0)\]"                # clear color
fi

if [[ -z ${PROMPT_FRONT_LOCAL-} ]]; then PROMPT_FRONT_LOCAL=""; fi
if [[ -z ${PROMPT_BACK_LOCAL-} ]]; then PROMPT_BACK_LOCAL=""; fi
PROMPT_FRONT="\
${USER_COLOR}\u${CLEAR}\
${AT_COLOR}@${CLEAR}\
${HOST_COLOR}\h${CLEAR} \
[${EXIT_STATUS_COLOR}\$?${CLEAR}]\
${PROMPT_FRONT_LOCAL}"
PROMPT_BACK=" \
${PROMPT_BACK_LOCAL}\
{${DATE_COLOR}\D{%F %A}${CLEAR} \
${TIME_COLOR}\t${CLEAR}} \
${PATH_COLOR}\w${CLEAR}\n\
${PROMPT_COLOR}\$${CLEAR} "

## Git
if source "${HOME}/.local/share/git-core/contrib/completion/git-prompt.sh" 2> /dev/null ||
    source /usr/lib/git-core/git-sh-prompt 2> /dev/null ||
    source /usr/share/git-core/contrib/completion/git-prompt.sh 2> /dev/null; then

    export GIT_PS1_SHOWDIRTYSTATE=true
    export GIT_PS1_SHOWSTASHSTATE=true
    export GIT_PS1_SHOWUNTRACKEDFILES=true
    export GIT_PS1_SHOWUPSTREAM="auto"
    export GIT_PS1_SHOWCOLORHINTS=true
    export GIT_PS1_SHOWCONFLICTSTATE=no

    # shellcheck disable=SC2016
    PROMPT_FRONT="${PROMPT_FRONT}"'$(__git_ps1)'
fi

PS1="${PROMPT_FRONT}${PROMPT_BACK}"

# Improved search backwards
if [[ -f "/usr/share/doc/fzf/examples/key-bindings.bash" ]]; then source /usr/share/doc/fzf/examples/key-bindings.bash; fi
if [[ -f "/usr/share/fzf/key-bindings.bash" ]]; then source /usr/share/fzf/key-bindings.bash; fi
# Searching files and other completions (for example with `cat **<Tab>`, and processes with `kill -9 <Tab>`)
if [[ -f "/usr/share/bash-completion/completions/fzf" ]]; then source /usr/share/bash-completion/completions/fzf; fi
if [[ -f "/usr/share/fzf/completion.bash" ]]; then source /usr/share/fzf/completion.bash; fi

# Aliases
if [[ -f "${HOME}/.bash_aliases" ]]; then source "${HOME}/.bash_aliases"; fi

# Loading completed successfully
true
