#!/usr/bin/env bash
set -eu

i3-msg border none

trap "tput cnorm" EXIT
tput civis

clear
while read -rn1 -s; do
    clear
done
