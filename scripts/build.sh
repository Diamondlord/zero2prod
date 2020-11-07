#!/usr/bin/env bash
#enables a mode of the shell where all executed commands are printed to the terminal
set -x
#The "set" lines These lines deliberately cause your script to fail.
set -eo pipefail

DOCKER_BUILDKIT=1 docker build --tag zero2prod --file Dockerfile .
