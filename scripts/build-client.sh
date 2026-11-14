#!/usr/bin/env bash
# Put the browser-side Leptos island (crates/client) into public/pkg/client:
#   scrov_client.js        ES module glue from wasm-bindgen
#   scrov_client_bg.wasm   the WebAssembly module
#
# When prebuilt/ matches the sources, its copy is used and no Rust toolchain
# is needed. Otherwise the client is compiled; FORCE_RUST_BUILD=1 always
# compiles.
#
# The wasm-bindgen CLI must match the wasm-bindgen crate in Cargo.lock exactly.
# The script looks for that version on PATH and in local caches, then downloads
# the prebuilt binary, and compiles it from source as a last resort.
set -euo pipefail
cd "$(dirname "$0")/.."
# shellcheck source=scripts/lib/prebuilt.sh
source scripts/lib/prebuilt.sh

out=public/pkg/client

if prebuilt_is_current && [ -f "$PREBUILT_DIR/client/scrov_client_bg.wasm" ]; then
	rm -rf "$out"
	mkdir -p "$(dirname "$out")"
	cp -R "$PREBUILT_DIR/client" "$out"
	echo "client: using the prebuilt copy"
	exit 0
fi

# shellcheck source=scripts/lib/rust-env.sh
source scripts/lib/rust-env.sh

version=$(awk '/^name = "wasm-bindgen"$/ { getline; gsub(/[^0-9.]/, "", $3); print $3; exit }' Cargo.lock)
if [ -z "$version" ]; then
	echo "could not read the wasm-bindgen version from Cargo.lock" >&2
	exit 1
fi

cache="target/wasm-bindgen/$version"

find_wasm_bindgen() {
	local candidate
	for candidate in \
		"$(command -v wasm-bindgen || true)" \
		"$cache/wasm-bindgen" \
		"$cache/bin/wasm-bindgen" \
		"$HOME"/.cache/worker-build/wasm-bindgen-*-"$version"/wasm-bindgen; do
		if [ -x "$candidate" ] && "$candidate" --version 2>/dev/null | grep -qF "$version"; then
			echo "$candidate"
			return 0
		fi
	done
	return 1
}

download_wasm_bindgen() {
	local triple
	case "$(uname -s)-$(uname -m)" in
	Linux-x86_64) triple=x86_64-unknown-linux-musl ;;
	Linux-aarch64 | Linux-arm64) triple=aarch64-unknown-linux-musl ;;
	Darwin-arm64) triple=aarch64-apple-darwin ;;
	Darwin-x86_64) triple=x86_64-apple-darwin ;;
	*) return 1 ;;
	esac
	local name="wasm-bindgen-$version-$triple"
	local url="https://github.com/wasm-bindgen/wasm-bindgen/releases/download/$version/$name.tar.gz"
	mkdir -p "$cache"
	echo "downloading wasm-bindgen $version ($triple)"
	curl --proto '=https' --tlsv1.2 -sSfL "$url" | tar -xz -C "$cache" --strip-components 1 "$name/wasm-bindgen"
}

if ! bindgen=$(find_wasm_bindgen); then
	if ! download_wasm_bindgen; then
		echo "no prebuilt wasm-bindgen $version for this platform: compiling it"
		cargo install -q wasm-bindgen-cli --version "$version" --locked --root "$cache"
	fi
	bindgen=$(find_wasm_bindgen)
fi

cargo build -p scrov-client --release --target wasm32-unknown-unknown

rm -rf "$out"
"$bindgen" --target web --no-typescript --remove-name-section --remove-producers-section \
	--out-dir "$out" --out-name scrov_client \
	target/wasm32-unknown-unknown/release/scrov_client.wasm
echo "client: compiled, $(wc -c <"$out/scrov_client_bg.wasm") bytes of wasm in $out"
