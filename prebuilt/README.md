# prebuilt

Copies of the two build outputs that need a Rust toolchain:

- `worker/` — the Cloudflare Worker bundle (`crates/worker`, built by `worker-build`)
- `client/` — the WebMCP client (`crates/client`, built by `wasm-bindgen`)
- `source-hash` — a hash of every file those builds read

They are committed so that deploying does not have to compile Rust. Cloudflare's
builders differ in what they can build: one of the accounts connected to this
repository cannot finish the Rust compile. With these files a deploy only
installs the npm packages, builds the CSS and uploads.

`scripts/build-worker.sh` and `scripts/build-client.sh` use these copies only
when `source-hash` matches the current sources (`Cargo.toml`, `Cargo.lock`,
`rust-toolchain.toml`, `crates/app`, `crates/client`, `crates/worker`,
`content` and the two build scripts). If anything changed they compile from
source instead, so a stale copy is never deployed.

Do not edit these files by hand. To refresh them:

```bash
bun run update-prebuilt
```

On `main` the `Prebuilt artifacts` workflow does this automatically whenever
the sources change, and commits the result.
