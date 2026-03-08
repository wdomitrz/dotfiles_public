#!/usr/bin/env sh
[ ! $# -eq 1 ] && return 1
destination="$1"

ssh-copy-id "${destination}" 2> /dev/null
scp -q "${HOME}"/.ssh/authorized_keys "${destination}":~/.ssh/authorized_keys
scp -q "${HOME}"/.bashrc "${HOME}"/.profile "${destination}":
