#!/usr/bin/env bash
# Make less more friendly for non-text input files (see lesspipe(1)).
[[ -x /usr/bin/lesspipe ]] && eval "$(SHELL=/bin/sh lesspipe)"

# enable color support of ls and also add handy aliases
if [[ -x /usr/bin/dircolors ]]; then
    (test -r "${HOME}/.dircolors" && eval "$(dircolors --bourne-shell "${HOME}/.dircolors")") || eval "$(dircolors --bourne-shell)"
    alias ls='ls --color=auto'
    alias dir='dir --color=auto'
    alias vdir='vdir --color=auto'

    alias grep='grep --color=auto'
    alias fgrep='fgrep --color=auto'
    alias egrep='egrep --color=auto'
fi

alias gla='git log --all --graph --oneline --decorate --date-order'

# let aliases work after sudo (see http://askubuntu.com/a/22043)
alias sudo='sudo '

# Add an "alert" alias for long running commands.  Use like so:
#   sleep 10; alert
alias alert='notify-send --urgency=low -i "$([ $? = 0 ] && echo terminal || echo error)" "$(history|tail -n1|sed --expression='\''s/^\s*[0-9]\+\s*//;s/[;&|]\s*alert$//'\'')" ; paplay /usr/share/sounds/freedesktop/stereo/message-new-instant.oga'

alias fzfcommandsearch="compgen -c | sort --unique | fzf --preview 'shopt -s expand_aliases; source ~/.bashrc; type {}'"

# some more ls aliases
alias ll='ls -l'
alias la='ls --almost-all -l'
alias l='ls -C'

alias pint='ping 8.8.8.8'

alias today='date --iso-8601'

alias emacs='emacsclient --create-frame --tty'

# Ignore errors
function I() {
    ("$@") 2>/dev/null
}
alias I='I '
# Run command independently
function DT() {
    ("$@") &
    disown
}
alias DT='DT '
[[ -z ${TERMINAL+variable_unset} ]] || alias nt='DT "${TERMINAL}"'
alias nv='nt -e nvim'

[[ "${TERM}" == "xterm-kitty" ]] && alias clear='printf "'"\E[H\E[3J"'"'
[[ "${TERM}" != "xterm-kitty" ]] && alias tmux='TERM=screen-256color tmux'

# Package manager aliases
alias pcin='sudo apt-get install'

alias pcud='sudo apt-get update'
alias pcug='sudo apt-get dist-upgrade'

alias pcar='sudo apt-get autoremove --purge'
alias pcpn='sudo apt-get purge'
alias pcum='sudo apt-mark auto'

alias pccl='sudo apt-get clean'
alias pccal='sudo apt-get autoclean'

alias pcsa='apt search'
alias pcsf='apt-file search'
alias pclf='apt-file list'

alias pcwhy='aptitude why'
alias pcwhynot='aptitude why-not'
alias pcdep='apt-cache depends'
alias pcdepi='apt-cache depends --installed'
alias pcrdep='apt-cache rdepends'
alias pcrdepi='apt-cache rdepends --installed'
alias pcsi='apt list --installed'
alias pcls='apt list'
alias pcsim='apt-mark showmanual'

alias pcyall='sudo apt-get update --yes && sudo apt-get dist-upgrade --yes && sudo apt-get autoremove --purge --yes'
