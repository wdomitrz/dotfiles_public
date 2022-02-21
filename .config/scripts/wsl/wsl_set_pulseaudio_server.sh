export PULSE_SERVER=tcp:$(grep nameserver /etc/resolv.conf | awk '{print $2}')
