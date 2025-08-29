#!/usr/bin/env bash
set -euo pipefail
set -x

source "${HOME}"/.profile
export DEBIAN_FRONTEND=noninteractive

"${HOME}"/.local/bin/install_scripts/install_packages.sh
"${HOME}"/.local/bin/install_scripts/install_global.sh
"${HOME}"/.local/bin/install_scripts/config_global.sh
"${HOME}"/.local/bin/install_scripts/install_user.sh
"${HOME}"/.local/bin/install_scripts/config_user.sh
if [[ -f "${HOME}"/.local/bin/install_scripts/machine_specific.sh ]]; then
  "${HOME}"/.local/bin/install_scripts/machine_specific.sh
fi
