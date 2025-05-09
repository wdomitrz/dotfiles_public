FROM debian

RUN apt-get update && apt-get install --yes --no-install-recommends git sudo

RUN apt-get install --yes --no-install-recommends bluetooth

# Mocks
RUN ln --force --symbolic /usr/bin/true /usr/local/sbin/update-grub && \
    ln --force --symbolic /usr/bin/true /usr/sbin/mkswap && \
    ln --force --symbolic /usr/bin/true /usr/sbin/swapon && \
    true

# Add a new user
RUN echo '%sudo ALL=(ALL) NOPASSWD:ALL' >> /etc/sudoers
RUN useradd --create-home --shell /bin/bash --groups sudo user
USER user
WORKDIR /home/user

COPY --chown=user:user "./.git" "./.git"
RUN git checkout -- .
RUN USER=user ./.local/bin/install_scripts/main.sh
