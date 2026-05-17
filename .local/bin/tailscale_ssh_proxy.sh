#!/usr/bin/env sh
set -eu

if [ "$#" -ne 2 ]; then
  echo "usage: $0 <host> <port>" >&2
  exit 1
fi

HOST="$1"
PORT="$2"

exec nc --proxy-type socks5 --proxy localhost:1055 "${HOST}" "${PORT}"
