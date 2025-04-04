FROM debian

RUN dpkg --add-architecture i386 && apt-get update && apt-get upgrade --yes

# Install ssh server and packages
ADD https://raw.githubusercontent.com/wdomitrz/dotfiles_public/refs/heads/main/.config/packages/packages.sorted.txt /
RUN cat /packages.sorted.txt | DEBIAN_FRONTEND=noninteractive xargs apt-get install --yes --no-install-recommends openssh-server && \
    rm /packages.sorted.txt

# Configure ssh
RUN mkdir /var/run/sshd && \
    (echo "ChallengeResponseAuthentication no" && \
        echo "PasswordAuthentication no" && \
        echo "PermitRootLogin no" && \
        echo "PubkeyAuthentication yes" && \
        echo "UsePAM yes" && \
        echo "X11Forwarding yes" && \
        echo "X11UseLocalhost no") | tee --append /etc/ssh/sshd_config

# Configure locales
RUN echo "locales locales/default_environment_locale select en_US.UTF-8" | debconf-set-selections && \
    echo "locales locales/locales_to_be_generated multiselect en_US.UTF-8 UTF-8, pl_PL.UTF-8 UTF-8" | debconf-set-selections && \
    rm --force --verbose "/etc/locale.gen" && \
    dpkg-reconfigure --frontend noninteractive locales

# Add a new user
RUN echo '%sudo ALL=(ALL) NOPASSWD:ALL' >> /etc/sudoers
RUN useradd --create-home --shell /bin/bash --groups sudo user
USER user
WORKDIR /home/user

# Setup user config
RUN touch ~/.Xauthority && mkdir --parents ./.ssh ./.local ./.cache ./.config
ADD --chown=user \
    https://raw.githubusercontent.com/wdomitrz/dotfiles_public/refs/heads/main/.bashrc \
    https://raw.githubusercontent.com/wdomitrz/dotfiles_public/refs/heads/main/.bash_aliases \
    https://raw.githubusercontent.com/wdomitrz/dotfiles_public/refs/heads/main/.profile \
    ./

# Add ssh
EXPOSE 22
ADD --chown=user https://github.com/wdomitrz.keys ./.ssh/authorized_keys
CMD sudo /usr/sbin/sshd -D

# Add ttyd
EXPOSE 7681/tcp
