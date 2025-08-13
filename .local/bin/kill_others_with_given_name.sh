#!/usr/bin/env bash
set -euo pipefail

pgrep_pattern="$1"

function find_my_ancestors() {
  sh -c 'pstree --ascii --show-pids --show-parents $$' | tr "-" "\n" |
    grep --only-matching --extended-regexp '\([0-9]+\)$' | tr -d "()"
}

(pgrep --full "${pgrep_pattern}" || true) |
  sort | comm -23 - <(find_my_ancestors | sort) |
  xargs --max-args=1 kill --signal KILL &> /dev/null
