// Starts the Rust client (crates/client), which registers the site's WebMCP
// tools on document.modelContext. Built by scripts/build-client.sh.
import init from "/pkg/client/scrov_client.js";

init().catch((error) => {
	console.warn("webmcp: the client failed to start", error);
});
