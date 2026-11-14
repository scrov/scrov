# Sourced by the build scripts. Makes sure `cargo` is the toolchain pinned in
# rust-toolchain.toml, installing rustup and the toolchain when they are
# missing (a bare CI or Cloudflare Workers Builds image).

if [ -d "$HOME/.cargo/bin" ]; then
	export PATH="$HOME/.cargo/bin:$PATH"
fi

if ! command -v rustup >/dev/null 2>&1; then
	echo "rustup not found: installing it"
	curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs |
		sh -s -- -y --profile minimal --default-toolchain none
	export PATH="$HOME/.cargo/bin:$PATH"
fi

# Installs the channel, components and targets named in rust-toolchain.toml.
# rustup 1.28+ does this with a bare `toolchain install`; older versions
# install on `rustup show`.
rustup toolchain install >/dev/null 2>&1 || rustup show active-toolchain >/dev/null
rustup target add wasm32-unknown-unknown >/dev/null 2>&1 || true
