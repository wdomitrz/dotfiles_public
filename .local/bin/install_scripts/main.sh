#!/usr/bin/env bash
set -euo pipefail
set -x

"${HOME}"/.local/bin/install_scripts/install_packages.sh
"${HOME}"/.local/bin/install_scripts/install_global.sh
"${HOME}"/.local/bin/install_scripts/config_global.sh
"${HOME}"/.local/bin/install_scripts/install_user.sh
"${HOME}"/.local/bin/install_scripts/config_user.sh
[[ -f "${HOME}"/.local/bin/install_scripts/machine_specific.sh ]] &&
    "${HOME}"/.local/bin/install_scripts/machine_specific.sh
