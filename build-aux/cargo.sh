#!/bin/bash
# Source: https://gitlab.gnome.org/World/podcasts/
set -ex

export OUTPUT="$2"
export CARGO_TARGET_DIR="$3"/target
export CARGO_HOME="$CARGO_TARGET_DIR"/cargo-home
export PROFILE="$4"

TARGET=debug
ARGS=()

if test "$PROFILE" != "Devel"; then
    echo "RELEASE MODE"
    ARGS+=('--release')
    TARGET=release
fi

if test -d vendor; then
    echo "VENDORED"
    ARGS+=('--frozen')
fi

cargo build ${ARGS[@]} --manifest-path="$1"/Cargo.toml &&
cp "$CARGO_TARGET_DIR"/${TARGET}/news_flash_gtk "$OUTPUT"

