#!/usr/bin/env bash

function install_multi_touch_gestures_fusuma() {
    gem install --user-install fusuma
}

function install_haskell_ghcup() {
    curl --proto '=https' --tlsv1.2 -sSf https://get-ghcup.haskell.org | sh
}

function install_nvim_appimage() {
    mkdir --parents "${HOME}"/.local/opt "${HOME}"/.local/bin
    save_dir="${HOME}"/.local/opt/nvim
    mkdir --parents "${save_dir}"
    save_file_path="${save_dir}"/nvim.appimage

    wget https://github.com/neovim/neovim/releases/download/stable/nvim.appimage --output-document="${save_file_path}"

    chmod +x "${save_file_path}"

    ln --symbolic --relative --force "${save_file_path}" "${HOME}"/.local/bin/nvim
}

function install_moonlight() {
    mkdir --parents "${HOME}"/.local/opt "${HOME}"/.local/bin "${HOME}"/.local/share/applications
    save_dir="${HOME}"/.local/opt/moonlight
    mkdir --parents "${save_dir}"
    save_file_path="${save_dir}"/moonlight.appimage

    wget https://github.com/moonlight-stream/moonlight-qt/releases/download/v5.0.1/Moonlight-5.0.1-x86_64.AppImage --output-document="${save_file_path}"
    wget https://raw.githubusercontent.com/moonlight-stream/moonlight-qt/master/app/deploy/linux/com.moonlight_stream.Moonlight.desktop --output-document="${save_dir}"/moonlight.desktop

    chmod +x "${save_file_path}"

    ln --symbolic --relative --force "${save_file_path}" "${HOME}"/.local/bin/moonlight
    ln --symbolic --relative --force "${save_dir}"/moonlight.desktop "${HOME}"/.local/share/applications
}

function use_system_nvim() {
    ln --symbolic /usr/bin/nvim "${HOME}"/.local/bin/ ||
        echo "nvim file/link already exists"
}

function install_kitty() {
    mkdir --parents "${HOME}"/.local/opt "${HOME}"/.local/bin "${HOME}"/.local/share/applications "${HOME}"/.local/share/icons
    curl --location https://sw.kovidgoyal.net/kitty/installer.sh |
        sh /dev/stdin launch=n dest="${HOME}"/.local/opt/
    ln --symbolic --relative --force "${HOME}"/.local/opt/kitty.app/bin/kitty "${HOME}"/.local/bin/
    ln --symbolic --relative --force "${HOME}"/.local/opt/kitty.app/share/applications/kitty.desktop "${HOME}"/.local/share/applications/
    ln --symbolic --relative --force "${HOME}"/.local/opt/kitty.app/share/icons/hicolor/256x256/apps/kitty.png "${HOME}"/.local/share/icons/
}

function install_blender() {
    mkdir --parents "${HOME}"/.local/opt "${HOME}"/.local/bin "${HOME}"/.local/share/applications "${HOME}"/.local/share/icons
    curl --location "https://mirror.clarkson.edu/blender/release/Blender3.6/blender-3.6.5-linux-x64.tar.xz" |
        tar --extract --xz --directory="${HOME}"/.local/opt/

    ln --symbolic --relative --force "${HOME}"/.local/opt/blender-3.6.5-linux-x64/blender "${HOME}"/.local/bin/
    ln --symbolic --relative --force "${HOME}"/.local/opt/blender-3.6.5-linux-x64/blender.desktop "${HOME}"/.local/share/applications/
    ln --symbolic --relative --force "${HOME}"/.local/opt/blender-3.6.5-linux-x64/blender*.svg "${HOME}"/.local/share/icons/
}

function install_fira_code() {
    fonts_dir="${HOME}/.local/share/fonts"
    if [[ ! -d "${fonts_dir}" ]]; then
        echo "mkdir --parents ${fonts_dir}"
        mkdir --parents "${fonts_dir}"
    else
        echo "Found fonts dir ${fonts_dir}"
    fi

    version=6.2
    zip="Fira_Code_v${version}.zip"
    curl --fail --location --show-error "https://github.com/tonsky/FiraCode/releases/download/${version}/${zip}" --output "${zip}"
    unzip -o -q -d "${fonts_dir}" "${zip}"
    rm "${zip}"

    echo "fc-cache -f"
    fc-cache -f
}

function download_ubuntu_wallpapers() {
    wallpapers_directory="${HOME}"/.local/share/backgrounds/ubuntu
    wallpapers_url="http://archive.ubuntu.com/ubuntu/pool/main/u/ubuntu-wallpapers/ubuntu-wallpapers_23.10.4.orig.tar.gz"
    mkdir --parents "${wallpapers_directory}"
    wget -O- "${wallpapers_url}" |
        tar --extract --ungzip --directory="${wallpapers_directory}"

    # Cleanup
    # Remove files that are not pictures
    find "${wallpapers_directory}" -type f -not -iname "*.png" -and -not -iname "*.jpg" -print0 |
        xargs --null rm
    # Remove backgrounds to remove
    backgrounds_to_remove_file="${HOME}/.config/backgrounds/backgrounds_to_remove.txt"
    [[ -f "${backgrounds_to_remove_file}" ]] &&
        xargs -L 1 find "${wallpapers_directory}" -name <"${backgrounds_to_remove_file}" |
        xargs rm
    # Remove empty directories
    find "${wallpapers_directory}" -type d -empty -print0 |
        xargs --null rmdir
}

function install_user_main() {
    set -euo pipefail
    set -x

    install_nvim_appimage
    install_multi_touch_gestures_fusuma
    download_ubuntu_wallpapers
}

if [[ "$#" -ne 1 ]] || [[ "${1}" != "--source-only" ]]; then
    install_user_main "${@}"
fi
