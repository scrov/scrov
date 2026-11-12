#!/usr/bin/env bash
# Build everything `wrangler deploy` uploads:
#   public/pkg, public/vendor, public/fonts   (bun: scripts/assets.ts + Tailwind)
#   crates/worker/build                       (worker-build: Rust → WebAssembly)
#
# Installs the Rust toolchain pieces it needs when they are missing, so it
# also works on a bare CI or Cloudflare Workers Builds image.
set -euo pipefail
cd "$(dirname "$0")/.."

if ! command -v cargo >/dev/null 2>&1; then
	if [ -x "$HOME/.cargo/bin/cargo" ]; then
		export PATH="$HOME/.cargo/bin:$PATH"
	else
		echo "cargo not found: installing a minimal Rust toolchain"
		curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs |
			sh -s -- -y --profile minimal --default-toolchain stable
		export PATH="$HOME/.cargo/bin:$PATH"
	fi
fi

rustup target add wasm32-unknown-unknown >/dev/null

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
