# If not running interactively, don't do anything
[[ $- != *i* ]] && return

[ -f "/etc/bashrc" ] && source /etc/bashrc

# Options
shopt -s autocd       # Automatically change directory
shopt -s checkwinsize # Detect resize
shopt -s extglob      # Use additional pattern matching features.
shopt -s globstar     # Enable **
shopt -s histappend   # Append to the history file
shopt -u failglob     # Unmatched patterns don't cause errors

export HISTFILE="$HOME/.cache/bash_history" # History file

HISTCONTROL=ignorespace # don't save lines with leading space
HISTSIZE=               # unlimited bash history
HISTFILESIZE=           # unlimited bash history

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

# Prompt
## Detect no color support
([ -x /usr/bin/tput ] && tput setaf 1 >&/dev/null) || NO_COLORS=1

if [ -z "${NO_COLORS}" ]; then
    # Trick to make colors work well in tmux and in other scenarios
    [ "$TERM" != "xterm-kitty" ] && TERM=xterm-256color

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

export PROMPT_FRONT="\
${USER_COLOR}\u${CLEAR}\
${AT_COLOR}@${CLEAR}\
${HOST_COLOR}\h${CLEAR} \
[${EXIT_STATUS_COLOR}\$?${CLEAR}]"
export PROMPT_BACK=" \
{${DATE_COLOR}\D{%F %A}${CLEAR} \
${TIME_COLOR}\t${CLEAR}} \
${PATH_COLOR}\w${CLEAR}\n\
${PROMPT_COLOR}\$${CLEAR} "

export PS1="${PROMPT_FRONT}${PROMPT_BACK}"

## Git
if source /usr/lib/git-core/git-sh-prompt 2>/dev/null ||
    source /usr/share/git-core/contrib/completion/git-prompt.sh 2>/dev/null; then

    export GIT_PS1_SHOWDIRTYSTATE=true     # staged '+', unstaged '*'
    export GIT_PS1_SHOWSTASHSTATE=true     # '$' something is stashed
    export GIT_PS1_SHOWUNTRACKEDFILES=true # '%' untracked files
    export GIT_PS1_SHOWUPSTREAM="auto"     # '<' behind, '>' ahead, '<>' diverged, '=' no difference
    export GIT_PS1_UNTRACKEDFILES=true
    export GIT_PS1_SHOWCOLORHINTS=true

    export PROMPT_COMMAND='__git_ps1 "${PROMPT_FRONT}" "${PROMPT_BACK}"'
fi

# enable programmable completion features
if ! shopt -oq posix; then
    if [ -f /usr/share/bash-completion/bash_completion ]; then
        source /usr/share/bash-completion/bash_completion
    elif [ -f /etc/bash_completion ]; then
        source /etc/bash_completion
    fi
fi

# Improved search backwards
[ -f "/usr/share/doc/fzf/examples/key-bindings.bash" ] && source /usr/share/doc/fzf/examples/key-bindings.bash
[ -f "/usr/share/fzf/key-bindings.bash" ] && source /usr/share/fzf/key-bindings.bash
# Searching files and other completions (for example with `cat **<Tab>`, and processes with `kill -9 <Tab>`)
[ -f "/usr/share/bash-completion/completions/fzf" ] && source /usr/share/bash-completion/completions/fzf
[ -f "/usr/share/fzf/completion.bash" ] && source /usr/share/fzf/completion.bash

# Aliases
[ -f "$HOME/.bash_aliases" ] && source "$HOME/.bash_aliases"
# Profile
[ -f "$HOME/.profile" ] && [ -z "${PROFILE_LOADED}" ] && source "$HOME/.profile"

# Loading completed successfully
true
