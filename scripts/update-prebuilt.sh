#!/usr/bin/env bash
# Compile the Worker and the WebMCP client from source and store the results
# in prebuilt/, with the hash of the sources they were built from.
#
#   bun run update-prebuilt                 always rebuild
#   bun run update-prebuilt -- --if-stale   rebuild only when the sources changed
set -euo pipefail
cd "$(dirname "$0")/.."
# shellcheck source=scripts/lib/prebuilt.sh
source scripts/lib/prebuilt.sh

if [ "${1:-}" = "--if-stale" ] && prebuilt_is_current; then
	echo "prebuilt/ is up to date"
	exit 0
fi

FORCE_RUST_BUILD=1 bash scripts/build-worker.sh

rm -rf "$PREBUILT_DIR/worker" "$PREBUILT_DIR/client"
mkdir -p "$PREBUILT_DIR/worker"
cp crates/worker/build/index.js crates/worker/build/index_bg.wasm "$PREBUILT_DIR/worker/"
cp -R public/pkg/client "$PREBUILT_DIR/client"
source_hash >"$PREBUILT_DIR/source-hash"
echo "prebuilt/ updated for source hash $(cat "$PREBUILT_DIR/source-hash")"
