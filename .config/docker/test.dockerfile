FROM debian

RUN apt-get update && apt-get install --yes --no-install-recommends ca-certificates git sudo

# Mock update-grub
RUN touch /usr/local/sbin/update-grub && chmod +x /usr/local/sbin/update-grub

# Add a new user
RUN echo '%sudo ALL=(ALL) NOPASSWD:ALL' >> /etc/sudoers
RUN useradd --create-home --shell /bin/bash --groups sudo user
USER user
WORKDIR /home/user

COPY --chown=user:user "./.git" "./.git"
RUN git checkout -- .
RUN . ./.profile && DEBIAN_FRONTEND=noninteractive USER=user ./.local/bin/install_scripts/main.sh
RUN . ./.profile && sanitize_synced_files.sh && git diff --exit-code
