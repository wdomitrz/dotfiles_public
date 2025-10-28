#!/usr/bin/env sh

[ -z "$1" ] && return
destination="$1"

ssh-copy-id "${destination}"
scp "${HOME}"/.ssh/authorized_keys "${destination}":~/.ssh/authorized_keys
scp \
  "${HOME}"/.bashrc "${HOME}"/.profile \
  "${HOME}"/.local/bin/.config_sshd.sh \
  "${destination}":
