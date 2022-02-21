#!/usr/bin/env bash
set -xu

# Nvidia drivers
sudo apt-get install --yes --install-recommends --no-install-suggests \
    nvidia-driver-510

# Install Chrome Remote Desktop
source ./install-global.sh --source-only
install_deb_from_url "https://dl.google.com/linux/direct/chrome-remote-desktop_current_amd64.deb"
