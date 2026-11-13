# Sourced by the build scripts. Support for prebuilt/, the committed copies of
# the two things that take a Rust toolchain to produce: the Worker bundle and
# the WebMCP client. See prebuilt/README.md.

PREBUILT_DIR=prebuilt

# One hash over every file the Rust builds read. Run from the repository root.
source_hash() {
	local tool="sha256sum"
	command -v sha256sum >/dev/null 2>&1 || tool="shasum -a 256"
	{
		printf '%s\n' Cargo.toml Cargo.lock rust-toolchain.toml \
			scripts/build-client.sh scripts/build-worker.sh
		find crates/app crates/client crates/worker content -type f \
			-not -path 'crates/worker/build/*' -not -name '.DS_Store'
	} | LC_ALL=C sort | tr '\n' '\0' | xargs -0 $tool | $tool | cut -d' ' -f1
}

# True when prebuilt/ was built from exactly these sources.
# FORCE_RUST_BUILD=1 always answers no, to compile from source.
prebuilt_is_current() {
	[ "${FORCE_RUST_BUILD:-0}" != "1" ] &&
		[ -f "$PREBUILT_DIR/source-hash" ] &&
		[ "$(cat "$PREBUILT_DIR/source-hash")" = "$(source_hash)" ]
}
