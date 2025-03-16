#!/usr/bin/env bash

sudo mkdir --parents /etc/ssh/sshd_config.d

echo -n "\
PermitRootLogin no
PasswordAuthentication no
ChallengeResponseAuthentication no
UsePAM no

PubkeyAuthentication yes

X11Forwarding yes
" | sudo tee /etc/ssh/sshd_config.d/my_sshd.conf

grep --quiet -E '^Include /etc/ssh/sshd_config.d/[\*,my_sshd].conf$' /etc/ssh/sshd_config || echo -n "

Include /etc/ssh/sshd_config.d/my_sshd.conf
" | sudo tee -a /etc/ssh/sshd_config

sudo systemctl restart sshd.service || sudo systemctl restart ssh.service
