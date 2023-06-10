#!/bin/sh

set -eu

EXEC=means-to-an-end
SERVER=computron

HERE="$(dirname "$0")"
cd "$HERE"

echo "Building"
cargo build --release
scp "target/release/$EXEC" "$SERVER:"

echo "Running"
ssh -t "$SERVER" -- "./$EXEC"
