#!/usr/bin/env bash

function install_multi_touch_gestures_fusuma() {
    gem install --user-install fusuma
}

function install_haskell_ghcup() {
    curl --fail --progress-bar --show-error --location --proto '=https' --tlsv1.2 https://get-ghcup.haskell.org | sh
}

function install_nvim() {
    source "${HOME}"/.local/bin/install_scripts/install_global.sh --source-only
    install_nvim_given_locations "${HOME}"/.local/opt "${HOME}"/.local/bin not_sudo
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
    unzip -q -d "${fonts_dir}" "${file_zip}"
    rm "${file_zip}"

    fc-cache -f
}

function install_jetbrains_mono() {
    wget_with_defaults.sh https://raw.githubusercontent.com/JetBrains/JetBrainsMono/master/install_manual.sh | bash
}

wallpapers_directory_base="${HOME}"/.local/share/backgrounds
function download_sequoia_wallpaper() {
    wget_with_defaults.sh 'https://drive.usercontent.google.com/download?id=1g9TyTAv2Xw_zyAOyRS_hAdI2wq3SU44q&export=download&authuser=0' |
        pv | sponge "${wallpapers_directory_base}"/backgrounds_both.dir/Sequoia-Sunrise.heic
}

function install_uv() {
    wget_with_defaults.sh --max-redirect=3 https://astral.sh/uv/install.sh | sh
}

function install_python_tools() {
    install_uv
    uv tool install basedpyright
    uv tool install ruff
}

function install_doc() {
    local url="$1"
    local name="$2"
    local target_dir="${HOME}/.local/share/doc/${name}"
    local package_file
    package_file="$(mktemp /tmp/XXXXXX.zip)"

    mkdir --parents "${target_dir}"
    wget_with_defaults.sh "${url}" > "${package_file}"
    unzip -qud "${target_dir}"/ "${package_file}"
    rm "${package_file}"
}

function install_python_doc() {
    ln --force --symbolic /usr/share/doc/python3 "${HOME}"/.local/share/doc/
    install_doc https://numpy.org/doc/1.24/numpy-html.zip "python3-numpy/html"
    install_doc https://pandas.pydata.org/pandas-docs/version/1.5/pandas.zip "python3-pandas/html"
}

function get_binary_from() {
    name="$1"
    checksum_sha256="$2"
    url="$3"
    shift 3

    file_path="${HOME}/.local/bin/${name}"
    wget_with_defaults.sh "$@" "${url}" > "${file_path}"
    if ! echo "${checksum_sha256} ${file_path}" | sha256sum --check; then
        echo "Wrong checksum for ${url}"
        return 1
    fi

    chmod +x "${file_path}"
}

function install_rmz_and_cpz() {
    get_binary_from rmz \
        "54f643c6ba170d613c65c48697000faf68d9c77611c10458ea5b1eac99799d25" \
        "https://github.com/SUPERCILEX/fuc/releases/download/3.0.1/x86_64-unknown-linux-gnu-rmz" \
        --max-redirect=1
    get_binary_from cpz \
        "cf8147eda901948c643975e3c29d4b10db9fbfdc475585d57f1446dfaa2fa16f" \
        "https://github.com/SUPERCILEX/fuc/releases/download/3.0.1/x86_64-unknown-linux-gnu-cpz" \
        --max-redirect=1
}

function install_nvim_plugins() {
    git submodule update --init --recursive
}

function install_user_main() {
    set -euo pipefail
    set -x

    install_python_tools
    install_rmz_and_cpz
    install_nvim_plugins
    install_python_doc
    download_sequoia_wallpaper
}

if [[ $# -ne 1 ]] || [[ ${1} != "--source-only" ]]; then
    install_user_main "${@}"
fi
