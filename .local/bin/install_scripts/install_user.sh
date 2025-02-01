#!/usr/bin/env bash

function install_multi_touch_gestures_fusuma() {
    gem install --user-install fusuma
}

function install_haskell_ghcup() {
    curl --fail --progress-bar --show-error --location --proto '=https' --tlsv1.2 https://get-ghcup.haskell.org | sh
}

function install_nvim_tar() {
    source "${HOME}"/.local/bin/install_scripts/install_global.sh --source-only
    install_nvim_tar_given_locations "${HOME}"/.local/opt "${HOME}"/.local/bin not_sudo
}

function install_moonlight() {
    mkdir --parents "${HOME}"/.local/opt "${HOME}"/.local/bin "${HOME}"/.local/share/applications
    save_dir="${HOME}"/.local/opt/moonlight
    mkdir --parents "${save_dir}"
    save_file_path="${save_dir}"/moonlight.appimage

    wget_with_defaults.sh https://github.com/moonlight-stream/moonlight-qt/releases/download/v5.0.1/Moonlight-5.0.1-x86_64.AppImage > "${save_file_path}"
    wget_with_defaults.sh https://raw.githubusercontent.com/moonlight-stream/moonlight-qt/master/app/deploy/linux/com.moonlight_stream.Moonlight.desktop > "${save_dir}"/moonlight.desktop

    chmod +x "${save_file_path}"

    ln --symbolic --relative --force "${save_file_path}" "${HOME}"/.local/bin/moonlight
    ln --symbolic --relative --force "${save_dir}"/moonlight.desktop "${HOME}"/.local/share/applications
}

function install_kitty() {
    mkdir --parents "${HOME}"/.local/opt "${HOME}"/.local/bin "${HOME}"/.local/share/applications "${HOME}"/.local/share/icons
    wget_with_defaults.sh https://sw.kovidgoyal.net/kitty/installer.sh |
        sh /dev/stdin launch=n dest="${HOME}"/.local/opt/
    ln --symbolic --relative --force "${HOME}"/.local/opt/kitty.app/bin/kitty "${HOME}"/.local/bin/
    ln --symbolic --relative --force "${HOME}"/.local/opt/kitty.app/share/applications/kitty.desktop "${HOME}"/.local/share/applications/
    ln --symbolic --relative --force "${HOME}"/.local/opt/kitty.app/share/icons/hicolor/256x256/apps/kitty.png "${HOME}"/.local/share/icons/
}

function install_blender() {
    mkdir --parents "${HOME}"/.local/opt "${HOME}"/.local/bin "${HOME}"/.local/share/applications "${HOME}"/.local/share/icons
    wget_with_defaults.sh "https://mirrors.dotsrc.org/blender/release/Blender4.0/blender-4.0.0-linux-x64.tar.xz" |
        tar --extract --xz --directory="${HOME}"/.local/opt/

    ln --symbolic --relative --force "${HOME}"/.local/opt/blender-4.0.0-linux-x64/blender "${HOME}"/.local/bin/
    ln --symbolic --relative --force "${HOME}"/.local/opt/blender-4.0.0-linux-x64/blender.desktop "${HOME}"/.local/share/applications/
    ln --symbolic --relative --force "${HOME}"/.local/opt/blender-4.0.0-linux-x64/blender*.svg "${HOME}"/.local/share/icons/
}

function install_fira_code() {
    fonts_dir="${HOME}/.local/share/fonts"
    mkdir --parents "${fonts_dir}"

    file_zip="$(mktemp /tmp/XXXXXX.zip)"
    wget_with_defaults.sh "https://github.com/tonsky/FiraCode/releases/download/6.2/Fira_Code_v6.2.zip" > "${file_zip}"
    unzip -d "${fonts_dir}" "${file_zip}"
    rm "${file_zip}"

    fc-cache -f
}

function install_jetbrains_mono() {
    wget_with_defaults.sh https://raw.githubusercontent.com/JetBrains/JetBrainsMono/master/install_manual.sh | bash
}

wallpapers_directory_base="${HOME}"/.local/share/backgrounds
function download_wallpapers_from_file_with_urls() {
    if [[ "$#" -ne 2 ]]; then
        echo "Expected 2 arguments: <output directory name> <urls path file>"
        return 1
    fi
    wallpapers_directory="${wallpapers_directory_base}"/"$1"
    wallpapers_urls_file="$2"
    mkdir --parents "${wallpapers_directory}"
    wget --no-verbose --show-progress \
        --directory-prefix="${wallpapers_directory}" \
        --input-file="${wallpapers_urls_file}"
}

function download_macos_wallpapers() {
    wallpapers_directory="${wallpapers_directory_base}"/macos
    wallpapers_directory_dark="${wallpapers_directory_base}"/backgrounds_dark.dir
    wallpapers_directory_light="${wallpapers_directory_base}"/backgrounds_light.dir
    download_wallpapers_from_file_with_urls macos "${HOME}"/.config/backgrounds/backgrounds_macos.sorted.txt
    ln -rsf "${wallpapers_directory}"/10-15-Night.jpg "${wallpapers_directory_dark}"/
    ln -rsf "${wallpapers_directory}"/10-15-Day.jpg "${wallpapers_directory_light}"/
}

function install_python_ruff() {
    pip3 install --user --upgrade ruff
}

function install_user_main() {
    set -euo pipefail
    set -x

    install_python_ruff
}

if [[ "$#" -ne 1 ]] || [[ "${1}" != "--source-only" ]]; then
    install_user_main "${@}"
fi
