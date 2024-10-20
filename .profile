#!/usr/bin/env sh
[ -z "${PROFILE_LOADED}" ] || return
PROFILE_LOADED=1

# set PATH so it includes user's private bin if it exists
[ -d "${HOME}/bin" ] && export PATH="${HOME}/bin:${PATH}"
[ -d "${HOME}/.local/bin" ] && export PATH="${HOME}/.local/bin:${PATH}"

# homebrew
[ -d "${HOME}/.linuxbrew" ] && eval "$("${HOME}/.linuxbrew/bin/brew" shellenv)"

# ghcup-env
[ -f "${HOME}/.ghcup/env" ] && . "${HOME}/.ghcup/env"

# Ruby user path
[ -f "/usr/bin/ruby" ] && GEM_HOME="$(ruby -e 'puts Gem.user_dir')/bin" && export PATH="${GEM_HOME}:${PATH}"

export EDITOR=vim
export VISUAL=vim
export ALTERNATE_EDITOR=""
export TERMINAL=kitty
export LANG="en_US.UTF-8"
export LANGUAGE="en_US"
export XSECURELOCK_SHOW_DATETIME=1
export XSECURELOCK_DATETIME_FORMAT="%F %A %T"
export XSECURELOCK_BLANK_TIMEOUT=0
export GTK_A11Y=none

[ -f "${HOME}/.config/local/profile.sh" ] && . "${HOME}/.config/local/profile.sh"

[ -n "${BASH_VERSION}" ] && [ -f "${HOME}/.bash_profile" ] && . "${HOME}/.bash_profile"
[ -n "${BASH_VERSION}" ] && [ -f "${HOME}/.bashrc" ] && . "${HOME}/.bashrc"

# Loading completed successfully
true
