// @ts-check

import { fileURLToPath, URL } from "node:url";
import { defineConfig } from "astro/config";
import mdx from "@astrojs/mdx";
import sitemap from "@astrojs/sitemap";
import tailwindcss from "@tailwindcss/vite";
import cloudflare from "@astrojs/cloudflare";

export default defineConfig({
	site: "https://scrov.app",

	integrations: [
		mdx(),
		sitemap(),
	],

	vite: {
		plugins: [
			tailwindcss(),
		],
		resolve: {
			alias: {
				"@": fileURLToPath(new URL("./src", import.meta.url)),
			},
		},
	},

	adapter: cloudflare({
		platformProxy: {
			enabled: true,
		},
	}),
});
