#!/usr/bin/env bash

function install_multi_touch_gestures_fusuma {
    gem install --user-install fusuma
}

function install_haskell_ghcup {
    curl --proto '=https' --tlsv1.2 -sSf https://get-ghcup.haskell.org | sh
}

function install_nvim_appimage {
    mkdir -p "$HOME"/.local/bin
    wget https://github.com/neovim/neovim/releases/download/stable/nvim.appimage -O "$HOME"/.local/bin/nvim
    chmod +x "$HOME"/.local/bin/nvim
}

function install_nvim_tar_gz {
    mkdir -p "$HOME"/.local/opt "$HOME"/.local/bin "$HOME"/.local/share/applications "$HOME"/.local/share/icons/hicolor/128x128/apps
    curl -L https://github.com/neovim/neovim/releases/download/stable/nvim-linux64.tar.gz | tar xz -C "$HOME"/.local/opt/
    ln -srf "$HOME"/.local/opt/nvim-linux64/bin/nvim "$HOME"/.local/bin/
    ln -srf "$HOME"/.local/opt/nvim-linux64/share/applications/nvim.desktop "$HOME"/.local/share/applications/
    ln -srf "$HOME"/.local/opt/nvim-linux64/share/icons/hicolor/128x128/apps/nvim.png "$HOME"/.local/share/icons/hicolor/128x128/apps/
}

function use_system_nvim {
    ln -s /usr/bin/nvim "$HOME"/.local/bin/ || echo "nvim file/link already exists"
}

function install_kitty {
    mkdir -p "$HOME"/.local/opt "$HOME"/.local/bin "$HOME"/.local/share/applications "$HOME"/.local/share/icons
    curl -L https://sw.kovidgoyal.net/kitty/installer.sh | sh /dev/stdin launch=n dest="$HOME"/.local/opt/
    ln -srf "$HOME"/.local/opt/kitty.app/bin/kitty "$HOME"/.local/bin/
    ln -srf "$HOME"/.local/opt/kitty.app/share/applications/kitty.desktop "$HOME"/.local/share/applications/
    ln -srf "$HOME"/.local/opt/kitty.app/share/icons/hicolor/256x256/apps/kitty.png "$HOME"/.local/share/icons/
}

function install_fira_code {
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

function enable_flathub {
    flatpak remote-add --user --if-not-exists flathub https://flathub.org/repo/flathub.flatpakrepo
}

function install_moonlight {
    flatpak install --user --assumeyes flathub com.moonlight_stream.Moonlight
    echo '\
#!/usr/bin/env sh
exec flatpak run com.moonlight_stream.Moonlight
' | tee "$HOME"/.local/bin/moonlight
    chmod +x "$HOME"/.local/bin/moonlight
}

function install_python_packages {
    ## Update pip
    pip3 install --user -U pip
    ## pip modules
    pip3 install --user -U \
        autopep8 \
        autotiling \
        ipython \
        matplotlib \
        nautilus-open-any-terminal \
        numpy \
        pandas \
        Pillow

}

function update_node_and_npm {
    npm install --global node@lts npm
}

function main {
    set -xue

    enable_flathub
    install_python_packages
    update_node_and_npm
    install_multi_touch_gestures_fusuma
    install_nvim_tar_gz
    install_kitty
    install_fira_code
    install_moonlight
}

if [ "$#" -ne 1 ] || [ "${1}" != "--source-only" ]; then
    main "${@}"
fi
