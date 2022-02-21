if [ -z "${NO_COLORS}" ]; then
    # Colors
    USER_COLOR="%F{10}"       # green
    AT_COLOR="%F{11}"         # yellow
    HOST_COLOR="%F{14}"       # cyan
    EXIT_STATUS_COLOR="%F{9}" # red
    DATE_COLOR="%F{208}"      # orange
    TIME_COLOR="%F{13}"       # magenta
    PATH_COLOR="%F{6}"        # light blue
    PROMPT_COLOR="%F{4}"      # blue
    CLEAR="%f"                # clear color
fi

export PROMPT_FRONT=""$(
)"${USER_COLOR}%n${CLEAR}"$(
)"${AT_COLOR}@${CLEAR}"$(
)"${HOST_COLOR}%M${CLEAR} "$(
)"[${EXIT_STATUS_COLOR}%?${CLEAR}]"
export PROMPT_BACK=""$(
)" {${DATE_COLOR}%D{%F %A}${CLEAR} "$(
)"${TIME_COLOR}%*${CLEAR}} "$(
)"${PATH_COLOR}%~${CLEAR}"$'\n'$(
)"${PROMPT_COLOR}\$${CLEAR} "

export PROMPT="${PROMPT_FRONT}${PROMPT_BACK}"
