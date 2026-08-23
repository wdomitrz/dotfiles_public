FROM debian:trixie

RUN apt-get update && \
    apt-get install --yes --no-install-recommends \
       git sudo openssh-server moreutils

# Configure ssh
RUN (echo "UsePAM yes" && cat /etc/ssh/sshd_config) | \
        sponge /etc/ssh/sshd_config

# Mocks
# For files owned by packages, divert them first so that apt upgrades
# (e.g. dist-upgrade of util-linux/systemd) do not replace the mocks.
RUN true && \
    ln --force --symbolic /usr/bin/true /usr/local/sbin/update-grub && \
    dpkg-divert --local --rename --divert /usr/bin/systemctl.distrib \
        /usr/bin/systemctl && \
    ln --symbolic /usr/bin/true /usr/bin/systemctl && \
    dpkg-divert --local --rename --divert /usr/sbin/mkswap.distrib \
        /usr/sbin/mkswap && \
    ln --symbolic /usr/bin/true /usr/sbin/mkswap && \
    dpkg-divert --local --rename --divert /usr/sbin/swapon.distrib \
        /usr/sbin/swapon && \
    ln --symbolic /usr/bin/true /usr/sbin/swapon && \
    true


# Add a new user
RUN echo '%sudo ALL=(ALL) NOPASSWD:ALL' >> /etc/sudoers && \
    useradd --create-home --shell /bin/bash --groups sudo user
USER user
WORKDIR /home/user

# Add ssh keys
RUN mkdir --parents ./.ssh
ADD --chown=user https://github.com/wdomitrz.keys ./.ssh/authorized_keys

# Add configs
COPY --chown=user:user "./.git" "./.git"
RUN git checkout -- .

RUN USER=user ./.local/bin/install_scripts/main.sh

EXPOSE 22
CMD sudo /usr/sbin/sshd -D
