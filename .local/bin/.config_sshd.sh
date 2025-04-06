#!/usr/bin/env bash

sudo mkdir --parents /etc/ssh/sshd_config.d

echo -n "\
PermitRootLogin no
PasswordAuthentication no
ChallengeResponseAuthentication no
UsePAM no

PubkeyAuthentication yes

X11Forwarding yes
" | sudo tee /etc/ssh/sshd_config.d/01_my_sshd.conf

sudo systemctl restart sshd.service || sudo systemctl restart ssh.service
