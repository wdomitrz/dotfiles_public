FROM debian

RUN apt-get update && \
    apt-get install --yes --no-install-recommends git sudo

# Install before the main set of packages to avoid dependency problems
RUN apt-get install --yes --no-install-recommends bluetooth

# Mocks
RUN ln --force --symbolic /usr/bin/true /usr/local/sbin/update-grub && \
    ln --force --symbolic /usr/bin/true /usr/sbin/mkswap && \
    ln --force --symbolic /usr/bin/true /usr/sbin/swapon && \
    true

# Add a new user
RUN echo '%sudo ALL=(ALL) NOPASSWD:ALL' >> /etc/sudoers && \
    useradd --create-home --shell /bin/bash --groups sudo user
USER user
WORKDIR /home/user

# Copy configs
COPY --chown=user:user "./.git" "./.git"
RUN git checkout -- .

RUN USER=user ./.local/bin/install_scripts/main.sh
