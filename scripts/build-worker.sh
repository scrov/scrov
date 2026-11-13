#!/usr/bin/env bash
# Build everything `wrangler deploy` uploads:
#   public/pkg, public/vendor, public/fonts   (bun run build:web: assets,
#                                              Tailwind, the WebMCP client)
#   crates/worker/build                       (worker-build: Rust → WebAssembly)
#
# Installs the Rust toolchain pieces it needs when they are missing, so it
# also works on a bare CI or Cloudflare Workers Builds image.
#
# On Cloudflare Workers Builds (WORKERS_CI is set) the builder has little disk
# and runs this script twice, once as the build command and once through
# wrangler's [build] hook. There the script frees intermediate files as it
# goes and skips the second run.
set -euo pipefail
cd "$(dirname "$0")/.."

stamp=target/.scrov-built
commit="${WORKERS_CI_COMMIT_SHA:-unknown}"
if [ -n "${WORKERS_CI:-}" ] && [ -f crates/worker/build/index.js ] &&
	[ "$(cat "$stamp" 2>/dev/null)" = "$commit" ]; then
	echo "already built for $commit"
	exit 0
fi

# shellcheck source=scripts/lib/rust-env.sh
source scripts/lib/rust-env.sh

# First, while the disk is still empty: compiling worker-build needs about
# 1 GB of scratch space, which is removed again right after.
if ! command -v worker-build >/dev/null 2>&1; then
	cargo install -q worker-build --version '^0.9' --locked --target-dir target/worker-build-install
	rm -rf target/worker-build-install
fi

if [ "${SKIP_WEB_BUILD:-0}" != "1" ]; then
	if [ ! -d node_modules ]; then
		bun install --frozen-lockfile
	fi
	bun run build:web
fi

if [ -n "${WORKERS_CI:-}" ]; then
	# The client is already copied into public/pkg/client; its intermediate
	# files are only in the way of the Worker build.
	rm -rf target/wasm32-unknown-unknown
fi

(cd crates/worker && worker-build --release)

if [ -n "${WORKERS_CI:-}" ]; then
	rm -rf target/wasm32-unknown-unknown
	mkdir -p target
	echo "$commit" >"$stamp"
fi
