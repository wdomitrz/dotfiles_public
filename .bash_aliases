# Make less more friendly for non-text input files (see lesspipe(1)).
[ -x /usr/bin/lesspipe ] && eval "$(SHELL=/bin/sh lesspipe)"

# enable color support of ls and also add handy aliases
if [ -x /usr/bin/dircolors ]; then
    test -r $HOME/.dircolors && eval "$(dircolors -b $HOME/.dircolors)" || eval "$(dircolors -b)"
    alias ls='ls --color=auto'
    alias dir='dir --color=auto'
    alias vdir='vdir --color=auto'

    alias grep='grep --color=auto'
    alias fgrep='fgrep --color=auto'
    alias egrep='egrep --color=auto'
fi

# colored GCC warnings and errors
#export GCC_COLORS='error=01;31:warning=01;35:note=01;36:caret=01;32:locus=01:quote=01'

alias gla='git log --all --graph --oneline --decorate'

# let aliases work after sudo (see http://askubuntu.com/a/22043)
alias sudo='sudo '

# Add an "alert" alias for long running commands.  Use like so:
#   sleep 10; alert
alias alert='notify-send --urgency=low -i "$([ $? = 0 ] && echo terminal || echo error)" "$(history|tail -n1|sed -e '\''s/^\s*[0-9]\+\s*//;s/[;&|]\s*alert$//'\'')" ; paplay /usr/share/sounds/freedesktop/stereo/message-new-instant.oga'

alias ix='curl -F '"'"'f:1=<-'"'"' ix.io'

alias copy='xclip -selection clipboard'

# some more ls aliases
alias ll='ls -Al'
alias la='ls -A'
alias l='ls -C'

alias pint='ping 8.8.8.8'

# Ignore errors
I() {
    ($@) 2>/dev/null
}
# Run command independently
DT() {
    ($@) &
    disown
}
alias nt='DT $TERMINAL'
alias nv='nt -e nvim'

# Not all machines support xterm-kitty
command -v kitty >/dev/null && alias ssh='TERM=xterm-256color ssh'

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
alias pcrdep='apt-cache rdepends'
alias pcsi='apt list --installed'
alias pcls='apt list'
alias pcsim='apt-mark showmanual'
