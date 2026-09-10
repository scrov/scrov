// @ts-check
import { defineConfig } from "astro/config";
import mdx from "@astrojs/mdx";
import sitemap from "@astrojs/sitemap";
import { fileURLToPath, URL } from "node:url";
import tailwindcss from "@tailwindcss/vite";
import cloudflare from "@astrojs/cloudflare";

// https://astro.build/config
export default defineConfig({
//*        vite: {
                plugins: [tailwindcss()],
                resolve: {
                        alias: {
                                "@": fileURLToPath(new URL("./src", import.meta.url)),
			},
		},
	},*/
	site: "https://scrov.app",
	integrations: [mdx(), sitemap()],
	adapter: cloudflare({
		platformProxy: {
			enabled: true,
		},
	}),
});

