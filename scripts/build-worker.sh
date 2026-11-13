#!/usr/bin/env bash
# Build everything `wrangler deploy` uploads:
#   public/pkg, public/vendor, public/fonts   (bun run build:web: assets,
#                                              Tailwind, the WebMCP client)
#   crates/worker/build                       (worker-build: Rust → WebAssembly)
#
# Installs the Rust toolchain pieces it needs when they are missing, so it
# also works on a bare CI or Cloudflare Workers Builds image.
set -euo pipefail
cd "$(dirname "$0")/.."
# shellcheck source=scripts/lib/rust-env.sh
source scripts/lib/rust-env.sh

if ! command -v worker-build >/dev/null 2>&1; then
	cargo install -q worker-build
fi

if [ "${SKIP_WEB_BUILD:-0}" != "1" ]; then
	if [ ! -d node_modules ]; then
		bun install --frozen-lockfile
	fi
	bun run build:web
fi

cd crates/worker
worker-build --release
