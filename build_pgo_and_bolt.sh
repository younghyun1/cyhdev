#!/bin/sh
set -eu

REPOSITORY_ROOT=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
cd "$REPOSITORY_ROOT"

export CYHDEV_OPT_LATEST_NIGHTLY=1
exec cargo run --quiet --locked --release --package xtask -- build-pgo-and-bolt "$@"
