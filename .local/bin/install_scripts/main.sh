#!/usr/bin/env bash
set -euo pipefail
set -x
source "${HOME}"/.local/bin/install_scripts/print_and_run.sh

source "${HOME}"/.profile

print_and_run "${HOME}"/.local/bin/install_scripts/install_packages.sh
print_and_run "${HOME}"/.local/bin/install_scripts/install_global.sh
print_and_run "${HOME}"/.local/bin/install_scripts/config_global.sh
print_and_run "${HOME}"/.local/bin/install_scripts/install_user.sh
print_and_run "${HOME}"/.local/bin/install_scripts/config_user.sh
if [[ -f "${HOME}"/.local/bin/install_scripts/machine_specific.sh ]]; then
  print_and_run "${HOME}"/.local/bin/install_scripts/machine_specific.sh
fi

# Sanitize and check if there is no change
print_and_run sanitize_synced_files.sh
print_and_run git diff --exit-code
