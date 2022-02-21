# source .profile
[ -f ~/.zprofile ] && source ~/.zprofile

# Set up the prompt
autoload -Uz promptinit
promptinit

# Options
## Enabled
### Use additional pattern matching features.
setopt extendedglob
### Append new history lines instead of overwriting.
setopt appendhistory
### Immediately report changes in background job status.
setopt notify
## Disabled
### Change directory given just path.
unsetopt autocd
### Unmatched patterns cause an error.
unsetopt nomatch
### Beep on errors.
unsetopt beep

# Key bindings
## Vim style keybindings
bindkey -v
bindkey jk vi-cmd-mode
## Search backward
bindkey "^R" history-incremental-search-backward
## Remove whole word
bindkey "^H" backward-kill-word

# History
HISTFILE=~/.cache/zsh_history
HISTSIZE=2097152
SAVEHIST=2097152

# Copy (in vi mode) to the system clipboard
source ~/.config/zsh/clipboard.sh

# Completion
zstyle :compinstall filename '~/.zshrc'
autoload -Uz compinit
compinit

# Prompt
source ~/.config/zsh/prompt.sh

# Aliases
source ~/.config/zsh/aliases.sh

# Git
source ~/.config/zsh/git.sh

# Use 256 colors
if [ "$TERM" != "xterm-kitty" ]; then
    export TERM=xterm-256color
fi

# Command not found
[ -r /etc/zsh_command_not_found ] && source /etc/zsh_command_not_found

# Syntax highlighting
[ -r /usr/share/zsh-syntax-highlighting/zsh-syntax-highlighting.zsh ] && source /usr/share/zsh-syntax-highlighting/zsh-syntax-highlighting.zsh
