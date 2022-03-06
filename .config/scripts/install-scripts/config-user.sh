#!/usr/bin/env sh
set -xue

# Add user to groups
sudo usermod -aG docker,input,kvm,lpadmin,audio,netdev,video,libvirt $USER

ln -s /usr/bin/nvim $HOME/.local/bin/ || echo "nvim file/link already exists"

# Use zsh as the default shell
chsh -s /usr/bin/zsh

# Set core file limit to 0
ulimit -c 0

# Create default directories
if [ -f "$HOME/.config/default_dirs/default_dirs.zip" ]; then
    cd $HOME
    unzip $HOME/.config/default_dirs/default_dirs.zip
    [ -f "$HOME/.config/gtk-3.0/bookmarks" ] || touch $HOME/.config/gtk-3.0/bookmarks
    for d in $(zipinfo -1 $HOME/.config/default_dirs/default_dirs.zip); do
        grep --quiet "$d" $HOME/.config/gtk-3.0/bookmarks || echo "file://$(pwd)/$d" \
            >>$HOME/.config/gtk-3.0/bookmarks
    done
    cd -
fi

# Open kitty from nautilus
glib-compile-schemas $HOME/.local/share/glib-2.0/schemas/
gsettings set com.github.stunkymonkey.nautilus-open-any-terminal terminal kitty

# Copy default transmission settings to the settings file
[ -f "$HOME/.config/transmission/settings.bck.json" ] && cp $HOME/.config/transmission/settings.bck.json $HOME/.config/transmission/settings.json

if !(crontab -l 2>/dev/null | grep --quiet set-background); then
    (
        crontab -l 2>/dev/null
        echo -n """
# Change background 1 minute
*/5 * * * * DISPLAY=:0 .local/bin/set-background
"""
    ) | crontab -
fi
