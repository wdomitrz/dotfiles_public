#!/usr/bin/env bash

function start_docker_server() {
    suffix="$1"
    if [[ -n "${suffix}" ]]; then
        suffix=_"${suffix}"
    fi
    docker_file="${HOME}"/.config/docker/my"${suffix}".dockerfile

    [[ -f "${docker_file}" ]] || (echo "No dockerfile '${docker_file}'file found" && exit 1)

    docker build --tag my"${suffix}":local - < "${docker_file}" &&
        echo "Starting ssh server. Press Ctrl-C to stop it" &&
        docker run --publish 2222:22 --tty=true --interactive=true my"${suffix}":local
}
export -f start_docker_server

if [[ "$#" -ne 1 ]] || [[ "${1}" != "--source-only" ]]; then
    tmux new -d "start_docker_server $*"
fi
