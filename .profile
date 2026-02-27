#!/usr/bin/env sh
if [ -n "${PROFILE_LOADED-}" ]; then return; fi
PROFILE_LOADED=1

# set PATH so it includes user's private bin if it exists
if [ -d "${HOME}/bin" ]; then export PATH="${HOME}/bin:${PATH}"; fi
if [ -d "${HOME}/.local/bin" ]; then export PATH="${HOME}/.local/bin:${PATH}"; fi

# homebrew
if [ -d "/home/linuxbrew/.linuxbrew" ]; then eval "$(/home/linuxbrew/.linuxbrew/bin/brew shellenv)"; fi
if [ -d "/opt/homebrew" ]; then eval "$(/opt/homebrew/bin/brew shellenv)"; fi
# ghcup
if [ -f "${HOME}/.ghcup/env" ]; then . "${HOME}/.ghcup/env"; fi

export EDITOR=vim
export VISUAL=vim
export MANPAGER='vim -c Man!'
export ALTERNATE_EDITOR=""
export TERMINAL=alacritty
export LANG="en_US.UTF-8"
export LC_CTYPE="${LANG}"
export LANGUAGE="en_US"
export XSECURELOCK_SHOW_DATETIME=1
export XSECURELOCK_DATETIME_FORMAT="%F %A %T"
export XSECURELOCK_BLANK_TIMEOUT=0
export XSECURELOCK_SAVER="${HOME}"/.local/bin/xsecurelock_saver_feh.sh
export GTK_A11Y=none
export HOMEBREW_NO_ANALYTICS=1

if [ -f "${HOME}/.local/bin/setup_scratch.sh" ]; then . "${HOME}/.local/bin/setup_scratch.sh"; fi

if [ -f "${HOME}/.config/local/profile.sh" ]; then . "${HOME}/.config/local/profile.sh"; fi

if [ -n "${BASH_VERSION-}" ] && [ -f "${HOME}/.bash_profile" ]; then . "${HOME}/.bash_profile"; fi
if [ -n "${BASH_VERSION-}" ] && [ -f "${HOME}/.bashrc" ]; then . "${HOME}/.bashrc"; fi

# Loading completed successfully
true
