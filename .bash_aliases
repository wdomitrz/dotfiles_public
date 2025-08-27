#!/usr/bin/env bash
# Make less more friendly for non-text input files (see lesspipe(1)).
if [[ -x /usr/bin/lesspipe ]]; then eval "$(SHELL=/bin/sh lesspipe)"; fi

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

function alert() {
  local -r stdout_file="$(mktemp)"
  local -r stderr_file="$(mktemp)"
  local -r start_time="$(date +%s.%N)"

  "$@" > >(tee "${stdout_file}") 2> >(tee "${stderr_file}" >&2)

  local -r exit_code="$?"
  local -r end_time="$(date +%s.%N)"

  local -r runtime_seconds=$(echo "${end_time} - ${start_time}" | bc -l)
  local -r runtime=$(date -d@0"${runtime_seconds}" -u +%H:%M:%S.%N | head -c 10)

  function cat_file_compact() {
    [[ $# -ne 1 ]] && return 1
    if [[ $(wc -l < "${stdout_file}") -gt 8 ]]; then
      head -n 4 "${stdout_file}" \
        && echo "..." \
        && tail -n 4 "${stdout_file}"
    else
      head "${stdout_file}"
    fi
  }
  export -f cat_file_compact
  local -r stdout="$(cat_file_compact "${stdout_file}")"
  local -r stderr="$(cat_file_compact "${stderr_file}")"
  rm --force "${stdout_file}" "${stderr_file}"

  notify-send "Finished [${exit_code}] after ${runtime}" "\$ $*
${stdout}
${stderr}
"
}
alias alert='alert '

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
  ("$@") 2> /dev/null
}
alias I='I '
# Run command independently
function DT() {
  ("$@") &
  disown
}
alias DT='DT '
if [[ -n ${TERMINAL-} ]]; then alias nt='DT "${TERMINAL}"'; fi
alias nv='nt -e nvim'

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

function pcyall() {
  sudo apt-get update --yes \
    && sudo apt-get dist-upgrade --yes \
    && sudo apt-get autoremove --purge --yes
}
