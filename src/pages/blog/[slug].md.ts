import { type CollectionEntry, getCollection } from "astro:content";
import type { APIRoute, GetStaticPaths } from "astro";
import { markdownResponse, postMarkdown } from "@/lib/agent-content";

export const prerender = true;

export const getStaticPaths: GetStaticPaths = async () => {
	const posts = await getCollection("blog");
	return posts.map((post) => ({ params: { slug: post.id }, props: { post } }));
};

export const GET: APIRoute<{ post: CollectionEntry<"blog"> }> = ({ props }) => {
	const { post } = props;
	return markdownResponse(
		postMarkdown({ id: post.id, ...post.data }, post.body ?? ""),
	);
};
