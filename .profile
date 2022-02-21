[ -f "$HOME/.bash_profile" ] && . "$HOME/.bash_profile"

# set PATH so it includes user's private bin if it exists
[ -d "$HOME/bin" ] && export PATH="$HOME/bin:$PATH"
[ -d "$HOME/.local/bin" ] && export PATH="$HOME/.local/bin:$PATH"

# homebrew
[ -d "$HOME/.linuxbrew" ] && eval $("$HOME/.linuxbrew/bin/brew" shellenv)

# ghcup-env
[ -f "$HOME/.ghcup/env" ] && . "$HOME/.ghcup/env"

# Ruby user path
[ -f "/usr/bin/ruby" ] && GEM_HOME="$(ruby -e 'puts Gem.user_dir')/bin" && export PATH="${GEM_HOME}:$PATH"

export EDITOR=vim
export VISUAL=vim
export TERMINAL=kitty
export QT_STYLE_OVERRIDE=adwaita-dark
export GTK_THEME=Adwaita:dark
export LANG="en_US.UTF-8"
export LANGUAGE="en_US"

[ -n "$BASH_VERSION" ] && [ -f "$HOME/.bashrc" ] && . "$HOME/.bashrc"
