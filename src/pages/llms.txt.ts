import { getCollection } from "astro:content";
import type { APIRoute } from "astro";
import { llmsTxt } from "@/lib/agent-content";

export const prerender = true;

export const GET: APIRoute = async () => {
	const posts = await getCollection("blog");
	const body = llmsTxt(posts.map((post) => ({ id: post.id, ...post.data })));
	return new Response(body, {
		headers: { "Content-Type": "text/plain; charset=utf-8" },
	});
};
