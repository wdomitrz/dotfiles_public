#!/usr/bin/env sh
echo 'Y' | sudo tee /sys/module/bluetooth/parameters/disable_ertm
