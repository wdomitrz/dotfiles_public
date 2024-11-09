#!/usr/bin/env bash

function start_docker_server() {
    docker build --tag my - < "${HOME}"/.config/docker/my.dockerfile &&
        echo "Starting ssh server. Press Ctrl-C to stop it" &&
        docker run --publish 2222:22 --tty=true --interactive=true my
}
export -f start_docker_server

tmux new -d "start_docker_server"
