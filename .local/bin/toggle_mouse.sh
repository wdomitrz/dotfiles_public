#!/usr/bin/env sh
ds=$(xinput list | grep -Eo 'id=[0-9]+\s+\[slave\s+pointer' | grep -Eo '[0-9]+')
for d in ${ds}; do
    echo "${d}"
    xinput list-props "${d}" | grep 'Device Enabled' | grep '0$' > /dev/null
    if [ $? -eq 1 ]; then
        xinput disable "${d}"
    else
        xinput enable "${d}"
    fi
done
xdotool mousemove 9999 0
