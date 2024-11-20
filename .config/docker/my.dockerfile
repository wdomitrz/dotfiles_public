FROM debian

RUN apt-get update && apt-get upgrade --yes
ADD https://raw.githubusercontent.com/wdomitrz/dotfiles_public/refs/heads/main/.config/packages/packages_base.sorted.txt /
RUN DEBIAN_FRONTEND=noninteractive apt-get install --yes --no-install-recommends openssh-server && \
    mkdir /var/run/sshd && \
    (echo "ChallengeResponseAuthentication no" && \
        echo "PasswordAuthentication no" && \
        echo "PermitRootLogin no" && \
        echo "PubkeyAuthentication yes" && \
        echo "UsePAM yes" && \
        echo "X11Forwarding yes" && \
        echo "X11UseLocalhost no") | tee --append /etc/ssh/sshd_config
RUN cat /packages_base.sorted.txt | DEBIAN_FRONTEND=noninteractive xargs apt-get install --yes --no-install-recommends && \
    rm /packages_base.sorted.txt
RUN echo "locales locales/default_environment_locale select en_US.UTF-8" | debconf-set-selections && \
    echo "locales locales/locales_to_be_generated multiselect en_US.UTF-8 UTF-8, pl_PL.UTF-8 UTF-8" | debconf-set-selections && \
    rm --force --verbose "/etc/locale.gen" && \
    dpkg-reconfigure --frontend noninteractive locales


RUN echo '%sudo ALL=(ALL) NOPASSWD:ALL' >> /etc/sudoers
RUN useradd --create-home --shell /bin/bash --groups sudo user
USER user
WORKDIR /home/user

RUN touch ~/.Xauthority && mkdir --parents ./.ssh ./.local ./.cache ./.config
ADD --chown=user \
    https://raw.githubusercontent.com/wdomitrz/dotfiles_public/refs/heads/main/.bashrc \
    https://raw.githubusercontent.com/wdomitrz/dotfiles_public/refs/heads/main/.bash_aliases \
    https://raw.githubusercontent.com/wdomitrz/dotfiles_public/refs/heads/main/.profile \
    ./
EXPOSE 22
ADD --chown=user https://github.com/wdomitrz.keys ./.ssh/authorized_keys

CMD sudo /usr/sbin/sshd -D
