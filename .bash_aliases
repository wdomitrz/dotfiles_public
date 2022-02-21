# Make less more friendly for non-text input files (see lesspipe(1)).
[ -x /usr/bin/lesspipe ] && eval "$(SHELL=/bin/sh lesspipe)"

# enable color support of ls and also add handy aliases
if [ -x /usr/bin/dircolors ]; then
    test -r ~/.dircolors && eval "$(dircolors -b ~/.dircolors)" || eval "$(dircolors -b)"
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

# Some package manager aliases
alias pcm='apt'

alias pcin='sudo pcm install'

alias pcrn='sudo pcm remove'
alias pcpn='sudo pcm purge'
alias pcar='sudo pcm autoremove --purge'
alias pcum='sudo apt-mark auto'

alias pcsa='pcm search'
alias pcsi='pcm list --installed'
alias pcls='pcm list'

alias pcud='sudo pcm update'
alias pcug='sudo pcm full-upgrade'

# Specific package manager alias
alias pcwhy='aptitude why'
alias pcdep='apt-cache depends'
alias pcrdep='apt-cache rdepends'
alias pcsf='apt-file search'
alias pclf='apt-file list'
