#!/usr/bin/env bash
# enable color support of ls and also add handy aliases
if [[ -x /usr/bin/dircolors ]]; then
  (test -r "${HOME}/.dircolors" && eval "$(dircolors --bourne-shell "${HOME}/.dircolors")") || eval "$(dircolors --bourne-shell)"
fi

alias ls='ls --color=auto'
alias grep='grep --color=auto'

# let aliases work after sudo (see http://askubuntu.com/a/22043)
alias sudo='sudo '
alias alert='alert '

if [[ ${TERM-} == "alacritty" ]]; then alias clear='clear && printf "'"\E[3J"'"'; fi

alias tempe='. ~/.local/bin/tempe'
