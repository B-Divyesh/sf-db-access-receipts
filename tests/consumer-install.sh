#!/bin/sh
# @claim:install-from-clean-checkout
set -eu

source_root=$(git rev-parse --show-toplevel)
consumer_root=$(mktemp -d)
cleanup() {
  rm -rf "$consumer_root"
}
trap cleanup EXIT HUP INT TERM

git clone --no-local "$source_root" "$consumer_root/source" >/dev/null
cargo install --path "$consumer_root/source" --root "$consumer_root/install" --locked >/dev/null
"$consumer_root/install/bin/db-receipts" --json demo > "$consumer_root/demo.json"
grep -q '"demo":true' "$consumer_root/demo.json"
