//! The HTML document around every page: head, theme CSS, scripts, footer.

use crate::{
    components::theme_toggle::ThemeToggle,
    config::{abs, config, font_family, themes},
    content::escape_html,
};
use leptos::prelude::*;

pub struct Meta<'a> {
    pub title: &'a str,
    pub description: Option<&'a str>,
    /// Site-relative path, used for the canonical URL.
    pub path: &'a str,
    /// Open Graph type: `website` or `article`.
    pub og_type: &'a str,
    /// Site-relative URL of this page's Markdown version, if it has one.
    pub markdown: Option<String>,
}

impl<'a> Meta<'a> {
    pub fn new(title: &'a str, path: &'a str) -> Self {
        Self {
            title,
            description: None,
            path,
            og_type: "website",
            markdown: None,
        }
    }
}

/// CSS generated from the distro list: which per-distro elements show, and
/// each theme's colours and font.
fn theme_css() -> String {
    let mut css = String::from("[data-distro]{display:none}");
    for slug in themes() {
        css.push_str(&format!(
            "[data-theme=\"{slug}\"] [data-distro=\"{slug}\"]{{display:revert}}"
        ));
    }
    for system in &config().systems {
        let c = &system.colors;
        css.push_str(&format!(
            "[data-theme=\"{}\"]{{--color-bg:{};--color-panel:{};--color-border:{};--color-fg:{};--color-dim:{};--color-accent:{};--color-c1:{};--color-c2:{};--color-c3:{};--font-mono:'{}',ui-monospace,'SF Mono',Menlo,Monaco,monospace}}",
            system.slug, c.bg, c.panel, c.border, c.fg, c.dim, c.accent, c.c1, c.c2, c.c3,
            font_family(&system.font),
        ));
    }
    css
}

fn head(meta: &Meta) -> String {
    let site = &config().site;
    let description = meta.description.unwrap_or(&site.description);
    let title = if meta
        .title
        .to_lowercase()
        .contains(&site.title.to_lowercase())
    {
        meta.title.to_string()
    } else {
        format!("{} · {}", meta.title, site.title)
    };
    let canonical = abs(meta.path);
    let image = abs(&site.image);

    let mut same_as = Vec::new();
    if !site.social.github.is_empty() {
        same_as.push(site.social.github.clone());
    }
    let json_ld = serde_json::json!({
        "@context": "https://schema.org",
        "@type": "WebSite",
        "name": site.title,
        "url": site.url,
        "description": description,
        "image": image,
        "author": {
            "@type": "Person",
            "name": site.author,
            "jobTitle": site.job_title,
            "url": site.url,
            "sameAs": same_as,
        },
    })
    .to_string()
    .replace("</", "<\\/");

    let themes_json = serde_json::to_string(&themes()).unwrap_or_else(|_| "[]".into());
    let e = escape_html;
    let markdown_link = meta
        .markdown
        .as_deref()
        .map(|href| {
            format!(
                "<link rel=\"alternate\" type=\"text/markdown\" href=\"{}\">",
                e(href)
            )
        })
        .unwrap_or_default();
    let twitter_creator = if site.twitter.is_empty() {
        String::new()
    } else {
        format!(
            "<meta name=\"twitter:creator\" content=\"{}\">",
            e(&site.twitter)
        )
    };

    format!(
        r#"<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<meta name="generator" content="scrov {version} (Rust, Leptos SSR)">
<link rel="icon" type="image/svg+xml" href="/favicon.svg">
<title>{title}</title>
<meta name="title" content="{title}">
<meta name="description" content="{description}">
<meta name="theme-color" content="{theme_color}">
<meta name="author" content="{author}">
<link rel="canonical" href="{canonical}">
<link rel="sitemap" type="application/xml" href="/sitemap-index.xml">
<link rel="alternate" type="application/rss+xml" title="{site_title}" href="/rss.xml">
{markdown_link}
<link rel="describedby" type="text/plain" title="llms.txt" href="/llms.txt">
<meta property="og:type" content="{og_type}">
<meta property="og:url" content="{canonical}">
<meta property="og:title" content="{title}">
<meta property="og:description" content="{description}">
<meta property="og:image" content="{image}">
<meta property="og:image:width" content="{image_width}">
<meta property="og:image:height" content="{image_height}">
<meta property="og:locale" content="{locale}">
<meta property="og:site_name" content="{site_title}">
<meta name="twitter:card" content="summary_large_image">
<meta name="twitter:url" content="{canonical}">
<meta name="twitter:title" content="{title}">
<meta name="twitter:description" content="{description}">
<meta name="twitter:image" content="{image}">
{twitter_creator}
<script type="application/ld+json">{json_ld}</script>
<script>(function(){{var t=localStorage.getItem("theme");if(t&&{themes_json}.indexOf(t)>=0)document.documentElement.dataset.theme=t}})()</script>
<style>{theme_css}</style>
<link rel="stylesheet" href="/pkg/fonts.css">
<link rel="stylesheet" href="/pkg/app.css">
<script defer src="/vendor/htmx.min.js"></script>
<script defer src="/vendor/hyperscript.min.js"></script>
<script defer src="/js/site.js"></script>
<script>(function(w,d,s,l,i){{w[l]=w[l]||[];w[l].push({{'gtm.start':new Date().getTime(),event:'gtm.js'}});var f=d.getElementsByTagName(s)[0],j=d.createElement(s),dl=l!='dataLayer'?'&l='+l:'';j.async=true;j.src='https://www.googletagmanager.com/gtm.js?id='+i+dl;f.parentNode.insertBefore(j,f)}})(window,document,'script','dataLayer','GTM-N9P4WDL5');</script>
<script async src="https://www.googletagmanager.com/gtag/js?id=G-7J9FFYLGYT"></script>
<script>window.dataLayer=window.dataLayer||[];function gtag(){{dataLayer.push(arguments)}}gtag('js',new Date());gtag('config','G-7J9FFYLGYT');</script>
<script async src="https://pagead2.googlesyndication.com/pagead/js/adsbygoogle.js?client=ca-pub-6431693003705528" crossorigin="anonymous"></script>"#,
        version = env!("CARGO_PKG_VERSION"),
        title = e(&title),
        description = e(description),
        theme_color = e(&site.theme_color),
        author = e(&site.author),
        canonical = e(&canonical),
        site_title = e(&site.title),
        og_type = e(meta.og_type),
        image = e(&image),
        image_width = site.image_width,
        image_height = site.image_height,
        locale = e(&site.locale),
        theme_css = theme_css(),
    )
}

#[component]
fn Footer() -> impl IntoView {
    let footer = &config().site.footer;
    view! {
        <footer class="mx-auto max-w-3xl mt-10 text-dim">
            <nav class="mb-3 flex flex-wrap gap-x-4 gap-y-1 text-[0.85rem]">
                <a class="hover:text-accent" href="/">"~"</a>
                <a class="hover:text-accent" href="/blog/">"blog/"</a>
                <a class="hover:text-accent" href="/lab/">"lab/"</a>
                <a class="hover:text-accent" href="/rss.xml">"rss.xml"</a>
            </nav>
            <div class="flex flex-wrap justify-between gap-2 text-[0.85rem]">
                <div class="flex flex-wrap items-center gap-2">
                    <span class="text-accent">"●"</span>
                    <span>{footer.status.clone()}</span>
                    <span class="text-border">"·"</span>
                    <span>{format!("{} errors", footer.errors)}</span>
                    <span class="text-border">"·"</span>
                    <span>{format!("{} warnings", footer.warnings)}</span>
                    <span class="text-border">"·"</span>
                    <span>{format!("uptime {}", footer.uptime)}</span>
                </div>
                <ThemeToggle />
            </div>
        </footer>
    }
}

/// Renders a complete HTML document around `body`.
pub fn page<V: IntoView>(meta: &Meta, body: impl FnOnce() -> V) -> String {
    let site = &config().site;
    let owner = Owner::new();
    let body_html = owner.with(|| {
        view! {
            <main class="mx-auto max-w-3xl">{body()}</main>
            <Footer />
        }
        .to_html()
    });

    format!(
        r#"<!doctype html>
<html lang="{lang}" data-theme="{theme}" class="scroll-smooth">
<head>
{head}
</head>
<body class="min-h-screen bg-bg p-6 font-mono text-sm sm:text-base leading-normal text-fg antialiased selection:bg-accent selection:text-bg md:py-12">
<noscript><iframe src="https://www.googletagmanager.com/ns.html?id=GTM-N9P4WDL5" height="0" width="0" style="display:none;visibility:hidden"></iframe></noscript>
{body_html}
</body>
</html>
"#,
        lang = escape_html(&site.locale),
        theme = escape_html(&site.default_theme),
        head = head(meta),
    )
}
