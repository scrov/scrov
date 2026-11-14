/**
 * Prepare the static assets the Rust site serves from public/:
 *
 *   public/vendor/   htmx, hyperscript, fengari-web and PGlite, copied from node_modules
 *   public/fonts/    latin woff2 files from @fontsource
 *   public/pkg/      fonts.css (app.css is written next to it by the Tailwind CLI)
 *   content/logos/   fastfetch ASCII logos, one per distro in content/config.json
 *
 * Logos are committed, so the Rust build never needs the network; this script
 * only downloads the ones that are missing.
 */
import {
	cpSync,
	existsSync,
	mkdirSync,
	readdirSync,
	writeFileSync,
} from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const modules = (path: string) => resolve(ROOT, "node_modules", path);
const out = (path: string) => resolve(ROOT, path);

function copy(from: string, to: string): void {
	mkdirSync(dirname(to), { recursive: true });
	cpSync(from, to);
}

// ── browser libraries ────────────────────────────────────────────────────────
copy(modules("htmx.org/dist/htmx.min.js"), out("public/vendor/htmx.min.js"));
copy(
	modules("hyperscript.org/dist/_hyperscript.min.js"),
	out("public/vendor/hyperscript.min.js"),
);
copy(
	modules("fengari-web/dist/fengari-web.js"),
	out("public/vendor/fengari-web.js"),
);

const pglite = modules("@electric-sql/pglite/dist");
for (const file of readdirSync(pglite)) {
	const wanted =
		file === "index.js" ||
		file === "pglite.wasm" ||
		file === "pglite.data" ||
		/^chunk-.*\.js$/.test(file);
	if (wanted) copy(resolve(pglite, file), out(`public/vendor/pglite/${file}`));
}

// ── fonts ────────────────────────────────────────────────────────────────────
const FONTS: { pkg: string; family: string; weights: number[] }[] = [
	{ pkg: "jetbrains-mono", family: "JetBrains Mono", weights: [400, 500, 700] },
	{ pkg: "ubuntu-mono", family: "Ubuntu Mono", weights: [400, 700] },
	{ pkg: "fira-code", family: "Fira Code", weights: [400, 500, 700] },
	{ pkg: "noto-sans-mono", family: "Noto Sans Mono", weights: [400, 500, 700] },
	{ pkg: "noto-sans", family: "Noto Sans", weights: [400, 500, 700] },
	{ pkg: "inter", family: "Inter", weights: [400, 500, 700] },
	{ pkg: "roboto-mono", family: "Roboto Mono", weights: [400, 500, 700] },
];

let fontCss = "";
for (const { pkg, family, weights } of FONTS) {
	for (const weight of weights) {
		const file = `${pkg}-latin-${weight}-normal.woff2`;
		copy(
			modules(`@fontsource/${pkg}/files/${file}`),
			out(`public/fonts/${file}`),
		);
		fontCss += `@font-face{font-family:"${family}";font-style:normal;font-weight:${weight};font-display:swap;src:url(/fonts/${file}) format("woff2")}\n`;
	}
}
mkdirSync(out("public/pkg"), { recursive: true });
writeFileSync(out("public/pkg/fonts.css"), fontCss);

// ── distro logos ─────────────────────────────────────────────────────────────
const LOGO_BASE =
	"https://raw.githubusercontent.com/fastfetch-cli/fastfetch/refs/heads/dev/src/logo/ascii";
const config = (await Bun.file(out("content/config.json")).json()) as {
	systems: { slug: string }[];
};
mkdirSync(out("content/logos"), { recursive: true });
for (const { slug } of config.systems) {
	const target = out(`content/logos/${slug}.txt`);
	if (existsSync(target)) continue;
	const response = await fetch(`${LOGO_BASE}/${slug.at(0)}/${slug}.txt`);
	if (!response.ok) {
		throw new Error(`logo for ${slug}: HTTP ${response.status}`);
	}
	writeFileSync(target, await response.text());
	console.log(`fetched logo: ${slug}`);
}

console.log("assets ready");
