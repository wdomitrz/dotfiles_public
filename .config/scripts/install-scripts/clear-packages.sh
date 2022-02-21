#!/usr/bin/env bash
set -xue

sudo apt-get autoremove --purge --yes \
    apport \
    dmz-cursor-theme \
    fonts-ubuntu \
    gdm3 \
    gedit \
    gnome-accessibility-themes \
    gnome-bluetooth \
    gnome-characters \
    gnome-logs \
    gnome-menus \
    gnome-session-bin \
    gnome-settings-daemon-common \
    gnome-shell \
    gnome-system-monitor \
    gnome-terminal \
    language-pack-gnome-en \
    plymouth-theme-* \
    seahorse \
    snapd \
    ubuntu-docs \
    ubuntu-minimal \
    ubuntu-mono \
    ubuntu-report \
    ubuntu-settings \
    ubuntu-sounds \
    whoopsie \
    xcursor-themes \
    yaru* \
    yelp ||
    echo "NOT UNINSTALLED"
