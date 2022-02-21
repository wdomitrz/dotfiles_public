#!/usr/bin/env bash

# Show commands before executing them and treat an unset variable as an error
# and stop if an error occurs
set -xue

cd ~/.config/scripts/install-scripts

./install-packages.sh
./clear-packages.sh
./install-global.sh
./config-global.sh
./install-user.sh
./config-user.sh
./machine-specific.sh

cd -
