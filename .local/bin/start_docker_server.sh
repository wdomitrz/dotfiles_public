#!/usr/bin/env sh
docker build --tag my - < "${HOME}"/.config/docker/my.dockerfile &&
    docker run --publish 2222:22 --tty=true --interactive=true my
