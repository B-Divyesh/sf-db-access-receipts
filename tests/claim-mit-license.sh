#!/bin/sh
# @claim:mit-license
set -eu

cargo package --allow-dirty >/dev/null
archive=$(find target/package -maxdepth 1 -type f -name 'db-access-receipts-*.crate' -print -quit)
test -n "$archive"
tar -xOf "$archive" --wildcards '*/LICENSE' | grep -qx 'MIT License'
