#!/usr/bin/env bash
function print_and_run() {
  printf '+'
  printf ' %q' "$@"
  printf '\n'
  "$@"
}
