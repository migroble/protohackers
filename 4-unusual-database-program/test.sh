#!/bin/sh

set -eu

EXEC=unusual-database-program
SERVER=computron

HERE="$(dirname "$0")"
cd "$HERE"

echo "Building"
cargo build --release
scp "target/release/$EXEC" "$SERVER:"

echo "Running"
ssh -t "$SERVER" -- "./$EXEC"
