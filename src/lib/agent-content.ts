import config from "@/config.json";

const { site, sections } = config;

/** Absolute URL for a site-relative path. */
export const abs = (path: string): string => new URL(path, site.url).href;

export interface AgentPost {
	id: string;
	title: string;
	description: string;
	pubDate: Date;
}

const byNewest = (a: AgentPost, b: AgentPost): number =>
	b.pubDate.valueOf() - a.pubDate.valueOf();

/** Markdown rendering of the home page, built from the same data as the HTML. */
export function homeMarkdown(): string {
	const { about, projects, uses, contact } = sections;
	const lines = [
		`# ${site.title}`,
		"",
		`> ${site.description}`,
		"",
		"## About",
		"",
		...about.bio.map((item) => `- ${item.text}`),
		"",
		"## Projects",
		"",
		...projects.projects.map((project) =>
			project.description
				? `- [${project.name}](${project.url}): ${project.description}`
				: `- [${project.name}](${project.url})`,
		),
		"",
		"## Uses",
		"",
		...uses.cpu.usage.map((item) => `- ${item.label}: ${item.detail}`),
		"",
		"## Contact",
		"",
		`- Email: <${contact.email}>`,
		`- GitHub: <${site.social.github}>`,
		"",
		"## More",
		"",
		`- [Blog](${abs("/blog/")})`,
		`- [RSS feed](${abs("/rss.xml")})`,
		`- [llms.txt](${abs("/llms.txt")})`,
		"",
	];
	return lines.join("\n");
}

/** llms.txt index (https://llmstxt.org) linking the Markdown version of each page. */
export function llmsTxt(posts: AgentPost[]): string {
	const lines = [
		`# ${site.title}`,
		"",
		`> ${site.description}`,
		"",
		"Each page is also available as Markdown at the URLs below.",
		"",
		"## Pages",
		"",
		`- [Home](${abs("/index.md")}): profile, projects, stack and contact`,
		"",
		"## Blog",
		"",
		...[...posts]
			.sort(byNewest)
			.map(
				(post) =>
					`- [${post.title}](${abs(`/blog/${post.id}.md`)}): ${post.description}`,
			),
		"",
		"## Optional",
		"",
		`- [RSS feed](${abs("/rss.xml")})`,
		`- [Sitemap](${abs("/sitemap-index.xml")})`,
		`- [Source code](${site.social.github})`,
		"",
	];
	return lines.join("\n");
}

/** Markdown rendering of one blog post from its source body. */
export function postMarkdown(post: AgentPost, body: string): string {
	return [
		`# ${post.title}`,
		"",
		`> ${post.description}`,
		"",
		`Published: ${post.pubDate.toISOString().slice(0, 10)}`,
		`Canonical: ${abs(`/blog/${post.id}/`)}`,
		"",
		body.trim(),
		"",
	].join("\n");
}

export const markdownResponse = (body: string): Response =>
	new Response(body, {
		headers: { "Content-Type": "text/markdown; charset=utf-8" },
	});
