#!/usr/bin/env bash

# Show commands before executing them and treat an unset variable as an error
# and stop if an error occurs
set -xue

cd "$HOME/.config/scripts/install_scripts"

./install_packages.sh
./clear_packages.sh
./install_global.sh
./config_global.sh
./install_user.sh
./config_user.sh
./machine_specific.sh

cd -
