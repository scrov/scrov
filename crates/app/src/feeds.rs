//! Machine-readable views of the site: Markdown twins, llms.txt, RSS, the
//! sitemap and the JSON the lab page seeds its databases from.

use crate::{
    config::{abs, config},
    content::{escape_html, posts, Post},
};
use serde_json::json;

/// Markdown version of the home page.
pub fn home_markdown() -> String {
    let cfg = config();
    let (site, sections) = (&cfg.site, &cfg.sections);
    let mut out = format!("# {}\n\n> {}\n\n## About\n\n", site.title, site.description);
    for item in &sections.about.bio {
        out.push_str(&format!("- {}\n", item.text));
    }
    out.push_str("\n## Projects\n\n");
    for project in &sections.projects.projects {
        if project.description.is_empty() {
            out.push_str(&format!("- [{}]({})\n", project.name, project.url));
        } else {
            out.push_str(&format!(
                "- [{}]({}): {}\n",
                project.name, project.url, project.description
            ));
        }
    }
    out.push_str("\n## Uses\n\n");
    for usage in &sections.uses.cpu.usage {
        out.push_str(&format!("- {}: {}\n", usage.label, usage.detail));
    }
    out.push_str(&format!(
        "\n## Contact\n\n- Email: <{}>\n",
        sections.contact.email
    ));
    if !site.social.github.is_empty() {
        out.push_str(&format!("- GitHub: <{}>\n", site.social.github));
    }
    out.push_str(&format!(
        "\n## More\n\n- [Blog]({})\n- [Lab]({})\n- [RSS feed]({})\n- [llms.txt]({})\n",
        abs("/blog/"),
        abs("/lab/"),
        abs("/rss.xml"),
        abs("/llms.txt"),
    ));
    out
}

/// Markdown version of the blog index.
pub fn blog_markdown() -> String {
    let mut out = format!("# {} blog\n\n", config().site.title);
    for post in posts() {
        out.push_str(&format!(
            "- {} — [{}]({}): {}\n",
            post.date.iso(),
            post.title,
            abs(&format!("/blog/{}.md", post.slug)),
            post.description
        ));
    }
    out
}

/// Markdown version of one post.
pub fn post_markdown(post: &Post) -> String {
    format!(
        "# {}\n\n> {}\n\nPublished: {}\nCanonical: {}\n\n{}\n",
        post.title,
        post.description,
        post.date.iso(),
        abs(&format!("/blog/{}/", post.slug)),
        post.body
    )
}

/// llms.txt index (https://llmstxt.org).
pub fn llms_txt() -> String {
    let site = &config().site;
    let mut out = format!(
        "# {}\n\n> {}\n\nEvery page is also available as Markdown: request it with `Accept: text/markdown`, or use the URLs below.\n\n## Pages\n\n- [Home]({}): profile, projects, stack and contact\n- [Blog]({}): list of posts\n\n## Blog\n\n",
        site.title,
        site.description,
        abs("/index.md"),
        abs("/blog.md"),
    );
    for post in posts() {
        out.push_str(&format!(
            "- [{}]({}): {}\n",
            post.title,
            abs(&format!("/blog/{}.md", post.slug)),
            post.description
        ));
    }
    out.push_str(&format!(
        "\n## Optional\n\n- [RSS feed]({})\n- [Sitemap]({})\n- [Site data as JSON]({})\n",
        abs("/rss.xml"),
        abs("/sitemap-index.xml"),
        abs("/api/data.json"),
    ));
    if !site.social.github.is_empty() {
        out.push_str(&format!("- [Source code]({})\n", site.social.github));
    }
    out
}

pub fn rss() -> String {
    let site = &config().site;
    let e = escape_html;
    let mut items = String::new();
    for post in posts() {
        let link = abs(&format!("/blog/{}/", post.slug));
        items.push_str(&format!(
            "<item><title>{}</title><link>{link}</link><guid isPermaLink=\"true\">{link}</guid><description>{}</description><pubDate>{}</pubDate></item>",
            e(&post.title),
            e(&post.description),
            post.date.rfc822(),
        ));
    }
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?><rss version=\"2.0\"><channel><title>{}</title><description>{}</description><link>{}</link><language>{}</language>{items}</channel></rss>",
        e(&site.title),
        e(&site.description),
        abs("/"),
        e(&site.locale),
    )
}

pub fn sitemap_index() -> String {
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?><sitemapindex xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\"><sitemap><loc>{}</loc></sitemap></sitemapindex>",
        abs("/sitemap-0.xml")
    )
}

pub fn sitemap() -> String {
    let mut urls = vec![abs("/"), abs("/blog/"), abs("/lab/")];
    urls.extend(
        posts()
            .iter()
            .map(|post| abs(&format!("/blog/{}/", post.slug))),
    );
    let entries: String = urls
        .iter()
        .map(|url| format!("<url><loc>{}</loc></url>", escape_html(url)))
        .collect();
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?><urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">{entries}</urlset>"
    )
}

/// Projects, posts and systems as JSON. The lab page loads this into PGlite
/// tables and Lua tables.
pub fn data_json() -> String {
    let cfg = config();
    json!({
        "projects": cfg.sections.projects.projects.iter().map(|p| json!({
            "name": p.name,
            "description": p.description,
            "repo": p.repo,
            "stars": p.stars.trim_start_matches('★').trim(),
            "updated": p.updated,
            "url": p.url,
        })).collect::<Vec<_>>(),
        "posts": posts().iter().map(|p| json!({
            "slug": p.slug,
            "title": p.title,
            "description": p.description,
            "date": p.date.iso(),
            "url": abs(&format!("/blog/{}/", p.slug)),
        })).collect::<Vec<_>>(),
        "systems": cfg.systems.iter().map(|s| json!({
            "slug": s.slug,
            "os": s.os,
            "kernel": s.kernel,
            "shell": s.shell,
            "pkg": s.pkg,
            "wm": s.wm,
            "terminal": s.terminal,
            "font": s.font,
            "accent": s.colors.accent,
        })).collect::<Vec<_>>(),
    })
    .to_string()
}
