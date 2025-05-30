#!/usr/bin/env sh

"${HOME}"/.local/bin/list_backgrounds.sh |
    shuf --head-count=1 |
    awk '{print "\"" $0 "\""}' |
    xargs feh --no-fehbg --bg-fill ||
    (
        THEME_FILE="${HOME}"/.config/theme/theme.txt
        if [ -f "${THEME_FILE}" ] && [ "$(cat "${THEME_FILE}")" = "light" ]; then
            xsetroot -grey
        else
            xsetroot -solid "#000000"
        fi
    ) || true
