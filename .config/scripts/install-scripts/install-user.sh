#!/usr/bin/env bash

function install-multi-touch-gestures-fusuma {
    gem install --user-install fusuma
}

function install-haskell-ghcup {
    curl --proto '=https' --tlsv1.2 -sSf https://get-ghcup.haskell.org | sh
}

function install-nvim-appimage {
    mkeir -p ~/.local/bin
    wget https://github.com/neovim/neovim/releases/download/stable/nvim.appimage -O ~/.local/bin/nvim
    chmod +x ~/.local/bin/nvim
}

function install-nvim-tar-gz {
    mkdir -p ~/.local/opt ~/.local/bin ~/.local/share/applications ~/.local/share/icons
    curl -L https://github.com/neovim/neovim/releases/download/stable/nvim-linux64.tar.gz | tar xz -C ~/.local/opt/
    ln -srf ~/.local/opt/nvim-linux64/bin/nvim ~/.local/bin/
    ln -srf ~/.local/opt/nvim-linux64/share/applications/nvim.desktop ~/.local/share/applications/
    ln -srf ~/.local/opt/nvim-linux64/share/pixmaps/nvim.png ~/.local/share/icons/
}

function install-kitty {
    mkdir -p ~/.local/opt ~/.local/bin ~/.local/share/applications ~/.local/share/icons
    curl -L https://sw.kovidgoyal.net/kitty/installer.sh | sh /dev/stdin launch=n dest=~/.local/opt/
    ln -srf ~/.local/opt/kitty.app/bin/kitty ~/.local/bin/
    ln -srf ~/.local/opt/kitty.app/share/applications/kitty.desktop ~/.local/share/applications/
    ln -srf ~/.local/opt/kitty.app/share/icons/hicolor/256x256/apps/kitty.png ~/.local/share/icons/
}

function install-fira-code {
    fonts_dir="${HOME}/.local/share/fonts"
    if [ ! -d "${fonts_dir}" ]; then
        echo "mkdir -p $fonts_dir"
        mkdir -p "${fonts_dir}"
    else
        echo "Found fonts dir $fonts_dir"
    fi

    version=5.2
    zip=Fira_Code_v${version}.zip
    curl --fail --location --show-error https://github.com/tonsky/FiraCode/releases/download/${version}/${zip} --output ${zip}
    unzip -o -q -d ${fonts_dir} ${zip}
    rm ${zip}

    echo "fc-cache -f"
    fc-cache -f
}

function install-moonlight-appimage {
    mkdir -p ~/.local/bin ~/.local/share/applications ~/.local/share/icons
    wget https://github.com/moonlight-stream/moonlight-qt/releases/download/v3.2.0/Moonlight-3.2.0-x86_64.AppImage -O ~/.local/bin/moonlight
    chmod +x ~/.local/bin/moonlight
    wget https://raw.githubusercontent.com/moonlight-stream/moonlight-qt/master/app/deploy/linux/com.moonlight_stream.Moonlight.desktop -O ~/.local/share/applications/moonlight.desktop
    wget https://raw.githubusercontent.com/moonlight-stream/moonlight-qt/master/app/moonlight_wix.png -O ~/.local/share/icons/moonlight.png
    sed -i "s|Icon=moonlight|Icon=$(ls ~/.local/share/icons/moonlight.png)|g" ~/.local/share/applications/moonlight.desktop
}

function install-youtube-music {
    mkdir -p ~/.local/share/applications ~/.local/share/icons
    echo -n """\
#!/usr/bin/env xdg-open
[Desktop Entry]
Version=1.0
Terminal=false
Type=Application
Name=YouTube Music
Exec=/opt/google/chrome/google-chrome -app=https://music.youtube.com/
Icon=youtube-music
""" | tee ~/.local/share/applications/youtube-music.desktop
    wget https://music.youtube.com/img/favicon_32.png -O ~/.local/share/icons/youtube-music.png
}

function main {
    set -xue

    # python pip
    ## Update pip
    pip3 install --user -U pip
    ## pip modules
    pip3 install --user -U \
        autopep8 \
        autotiling \
        matplotlib \
        nautilus-open-any-terminal \
        numpy \
        pandas \
        Pillow

    # nodejs npm
    ## Update node and npm
    npm install --global node@lts npm

    # Custom installs
    install-multi-touch-gestures-fusuma
    install-nvim-tar-gz
    install-kitty
    install-fira-code
    install-moonlight-appimage
    install-youtube-music
}

if [ "$#" -ne 1 ] || [ "${1}" != "--source-only" ]; then
    main "${@}"
fi
