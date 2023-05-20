#!/bin/sh
set +e
diesel setup
diesel migration run
set -e

exec $@
