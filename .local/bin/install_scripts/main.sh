#!/usr/bin/env bash
set -euo pipefail
set -x

cd "${HOME}/.local/bin/install_scripts"

./install_packages.sh
./clear_packages.sh
./install_global.sh
./config_global.sh
./install_user.sh
./config_user.sh
./machine_specific.sh

cd -
