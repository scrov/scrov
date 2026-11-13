#!/usr/bin/env bash
# Build everything `wrangler deploy` uploads:
#   public/pkg, public/vendor, public/fonts   (bun run build:web: assets,
#                                              Tailwind, the WebMCP client)
#   crates/worker/build                       (the Worker: Rust → WebAssembly)
#
# When prebuilt/ matches the sources, the Worker and the client are copied
# from there and no Rust toolchain is needed: that is the normal case on
# Cloudflare's builders. Otherwise both are compiled, installing rustup, the
# pinned toolchain and worker-build when they are missing. FORCE_RUST_BUILD=1
# always compiles.
set -euo pipefail
cd "$(dirname "$0")/.."
# shellcheck source=scripts/lib/prebuilt.sh
source scripts/lib/prebuilt.sh

build_web() {
	if [ "${SKIP_WEB_BUILD:-0}" != "1" ]; then
		if [ ! -d node_modules ]; then
			bun install --frozen-lockfile
		fi
		bun run build:web
	fi
}

if prebuilt_is_current && [ -f "$PREBUILT_DIR/worker/index.js" ]; then
	build_web
	rm -rf crates/worker/build
	cp -R "$PREBUILT_DIR/worker" crates/worker/build
	echo "worker: using the prebuilt copy"
	exit 0
fi

# shellcheck source=scripts/lib/rust-env.sh
source scripts/lib/rust-env.sh

# Compiling worker-build needs about 600 MB of scratch space, which is removed
# again right after.
if ! command -v worker-build >/dev/null 2>&1; then
	cargo install -q worker-build --version '^0.9' --locked --target-dir target/worker-build-install
	rm -rf target/worker-build-install
fi

build_web

(cd crates/worker && worker-build --release)
echo "worker: compiled"
