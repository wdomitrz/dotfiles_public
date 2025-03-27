#!/usr/bin/env bash

function start_docker_server_main() {
    suffix=""
    rebuild=false
    while [[ $# -gt 0 ]]; do
        case $1 in
        --suffix) suffix="$2" && shift 2 ;;
        --rebuild) rebuild="$2" && shift 2 ;;
        *) echo "Unknown $1" && return 1 ;;
        esac
    done

    if [[ -n ${suffix} ]]; then
        suffix=_"${suffix}"
    fi
    docker_file="${HOME}"/.config/docker/my"${suffix}".dockerfile

    if [[ ! -f ${docker_file} ]]; then
        echo "No dockerfile '${docker_file}'file found"
        return 1
    fi

    if "${rebuild}"; then
        docker build --tag my"${suffix}":local - < "${docker_file}" || return 1
    fi

    echo "Starting ssh server. Press Ctrl-C to stop it"
    docker run \
        --publish 2222:22 \
        --publish 7681:7681 \
        --tty=true --interactive=true \
        my"${suffix}":local
}
export -f start_docker_server_main

if [[ $# -ne 1 ]] || [[ ${1} != "--source-only" ]]; then
    tmux new -d "start_docker_server_main $*"
fi
