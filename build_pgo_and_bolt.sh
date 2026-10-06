#!/bin/sh
set -eu

REPOSITORY_ROOT=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
cd "$REPOSITORY_ROOT"

export CYHDEV_OPT_LATEST_NIGHTLY=1
if [ "$#" -eq 0 ]; then
    set -- target/optimization-inputs/config.json
fi
exec cargo xtask optimize run "$@"
