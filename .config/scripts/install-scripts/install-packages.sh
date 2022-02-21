#!/usr/bin/env bash
set -xue

# apt configuration files
sudo cp ~/.config/apt.conf.d/* /etc/apt/apt.conf.d/

# Enable 32 bit architecture
sudo dpkg --add-architecture i386

sudo apt-get update --yes
sudo apt-get dist-upgrade --yes

sudo apt-get install --yes --no-install-recommends \
    acpi \
    acpi-call-dkms \
    adwaita-icon-theme \
    adwaita-icon-theme-full \
    adwaita-qt \
    amd64-microcode \
    apt \
    apt-file \
    aptitude \
    apt-transport-https \
    arandr \
    bash \
    bash-completion \
    blueman \
    bolt \
    brightnessctl \
    build-essential \
    cheese \
    clangd \
    cmake \
    command-not-found \
    compton \
    cups \
    curl \
    desktop-file-utils \
    dex \
    dmenu \
    docker-compose \
    docker.io \
    drawing \
    dunst \
    eog \
    ethtool \
    evince \
    e-wrapper \
    exfat-fuse \
    exfat-utils \
    extlinux \
    file \
    file-roller \
    firefox \
    flameshot \
    fonts-arphic-ukai \
    fonts-arphic-uming \
    fonts-dejavu \
    fonts-firacode \
    fonts-noto-color-emoji \
    fwupd \
    gdb \
    geoclue-2.0 \
    git \
    gnome-calculator \
    gnome-disk-utility \
    gnome-keyring \
    gnome-multi-writer \
    gnome-screenshot \
    gnome-sound-recorder \
    gnome-themes-extra \
    gparted \
    gpg \
    gpm \
    grub2 \
    gzip \
    htop \
    hwinfo \
    i3 \
    i3lock \
    i3status \
    ibus-pinyin \
    intel-microcode \
    iputils-ping \
    ipython3 \
    kazam \
    kdeconnect \
    kitty \
    libgtk-3-bin \
    libinput-tools \
    lightdm \
    lightdm-gtk-greeter \
    linux-generic \
    linux-headers-generic \
    lxappearance \
    lxpolkit \
    mobile-broadband-provider-info \
    modemmanager \
    nautilus \
    nautilus-admin \
    nautilus-gtkhash \
    nautilus-image-converter \
    neovim \
    network-manager \
    network-manager-gnome \
    nitrogen \
    nmap \
    notify-osd \
    npm \
    ntfs-3g \
    onboard \
    openconnect \
    openssh-client \
    pandoc \
    pavucontrol \
    pdftk \
    playerctl \
    preload \
    printer-driver-all \
    progress \
    pulseaudio \
    pulseaudio-module-bluetooth \
    pv \
    python3 \
    python3-dev \
    python3-nautilus \
    python3-pip \
    python3-venv \
    python-is-python3 \
    redshift-gtk \
    rename \
    rofi \
    rsync \
    ruby \
    simplescreenrecorder \
    software-properties-common \
    software-properties-gtk \
    speedtest-cli \
    sshfs \
    sshuttle \
    sudo \
    syslinux \
    system-config-printer \
    tar \
    tesseract-ocr \
    tesseract-ocr-chi-sim \
    thinkfan \
    thunar \
    tlp \
    tmux \
    tpb \
    tp-smapi-dkms \
    transmission \
    tree \
    ttf-mscorefonts-installer \
    universal-ctags \
    unzip \
    update-notifier \
    usb-creator-gtk \
    vim-tiny \
    vlc \
    wakeonlan \
    wget \
    wine \
    wine32 \
    winetricks \
    x11-xkb-utils \
    xcape \
    xclip \
    xdotool \
    xfce4-notifyd \
    xfce4-power-manager \
    xfce4-taskmanager \
    xinit \
    xinput \
    xorg \
    xournal \
    xss-lock \
    xterm \
    zathura \
    zip \
    zsh \
    zsh-syntax-highlighting

sudo apt-get install --yes --install-recommends \
    qemu-kvm \
    qemu-system-x86 \
    virt-manager

sudo apt-get autoremove --purge --yes \
    ubuntu-wallpapers
sudo apt-get install --yes --install-suggests \
    gnome-backgrounds \
    ubuntu-wallpapers
