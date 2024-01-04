#!/usr/bin/env bash
set -euo pipefail
set -x

source "${HOME}"/.local/bin/install_scripts/clear_packages.sh --source-only
source "${HOME}"/.local/bin/install_scripts/config_global.sh --source-only
source "${HOME}"/.local/bin/install_scripts/config_user.sh --source-only
source "${HOME}"/.local/bin/install_scripts/install_global.sh --source-only
source "${HOME}"/.local/bin/install_scripts/install_packages.sh --source-only
source "${HOME}"/.local/bin/install_scripts/install_user.sh --source-only
#source "${HOME}"/.local/bin/install_scripts/machine_specific.sh --source-only

install_packages_main
clear_packages_main
install_global_main
# part of config_global_main which is not used in install_packages_main
config_global_rest
install_user_main
config_user_main
#machine_specific_main
