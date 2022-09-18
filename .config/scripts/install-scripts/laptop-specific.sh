#!/usr/bin/env bash

function configure_external_power_sleep {
    sudo sed -i -E "s/^#?(HandleLidSwitchExternalPower)\s*=\s*[a-z]*/\1=lock/g" /etc/systemd/logind.conf
    sudo sed -i -E "s/^#?(HandleLidSwitchDocked)\s*=\s*[a-z]*/\1=lock/g" /etc/systemd/logind.conf
}

function main {
    set -xue

    configure_external_power_sleep
}

if [ "$#" -ne 1 ] || [ "${1}" != "--source-only" ]; then
    main "${@}"
fi
