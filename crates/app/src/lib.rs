//! The scrov site.
//!
//! Pages are Leptos components rendered to HTML on the server; the fragments
//! htmx swaps in are written with shtml. [`handle`] is the whole router. It
//! takes a plain [`Request`] and returns a plain [`Response`], so the same
//! code runs on Cloudflare Workers (wasm), on the native server behind
//! Pingora, and inside the Tauri shell.

pub mod components;
pub mod config;
pub mod content;
pub mod feeds;
pub mod fragments;
pub mod http;
pub mod layout;
pub mod pages;

pub use http::{Request, Response};

use http::{HTML, JSON, MARKDOWN, RSS, TEXT, XML};

const HOME_LINKS: &str = "</llms.txt>; rel=\"describedby\"; type=\"text/plain\", </index.md>; rel=\"alternate\"; type=\"text/markdown\", </sitemap-index.xml>; rel=\"sitemap\"; type=\"application/xml\", </rss.xml>; rel=\"alternate\"; type=\"application/rss+xml\"";
const PAGE_LINKS: &str = "</llms.txt>; rel=\"describedby\"; type=\"text/plain\", </sitemap-index.xml>; rel=\"sitemap\"; type=\"application/xml\"";

fn html(body: String) -> Response {
    // Pages can be served as HTML or Markdown from the same URL.
    Response::ok(HTML, body)
        .header("Vary", "Accept")
        .header("Cache-Control", "public, max-age=300")
}

fn markdown(body: String) -> Response {
    Response::ok(MARKDOWN, body)
        .header("Vary", "Accept")
        .header("X-Robots-Tag", "noindex")
}

fn fragment(body: String) -> Response {
    Response::ok(HTML, body).header("X-Robots-Tag", "noindex")
}

fn not_found(path: &str) -> Response {
    Response::new(404, HTML, pages::not_found::render(path))
}

/// Routes one request.
pub fn handle(request: &Request) -> Response {
    if !matches!(request.method, "GET" | "HEAD") {
        return Response::new(405, TEXT, "Method not allowed").header("Allow", "GET, HEAD");
    }

    let path = request.path;
    match path {
        "/" | "/index.html" => {
            if request.prefers_markdown() {
                markdown(feeds::home_markdown())
            } else {
                html(pages::home::render()).header("Link", HOME_LINKS)
            }
        }
        "/index.md" => markdown(feeds::home_markdown()),
        "/blog" => Response::redirect("/blog/"),
        "/blog/" => {
            if request.prefers_markdown() {
                markdown(feeds::blog_markdown())
            } else {
                html(pages::blog::index()).header("Link", PAGE_LINKS)
            }
        }
        "/blog.md" => markdown(feeds::blog_markdown()),
        "/lab" => Response::redirect("/lab/"),
        "/lab/" => html(pages::lab::render()).header("Link", PAGE_LINKS),
        "/about" | "/about/" => Response::redirect("/#about"),
        "/llms.txt" => Response::ok(TEXT, feeds::llms_txt()).header("X-Robots-Tag", "noindex"),
        "/rss.xml" => Response::ok(RSS, feeds::rss()),
        "/sitemap-index.xml" => Response::ok(XML, feeds::sitemap_index()),
        "/sitemap-0.xml" => Response::ok(XML, feeds::sitemap()),
        "/api/data.json" => {
            Response::ok(JSON, feeds::data_json()).header("Cache-Control", "public, max-age=300")
        }
        "/healthz" => Response::ok(TEXT, "ok"),
        "/fragments/posts" => fragment(fragments::post_list(
            &request.param("q").unwrap_or_default(),
        )),
        _ => route_dynamic(request).unwrap_or_else(|| not_found(path)),
    }
}

fn route_dynamic(request: &Request) -> Option<Response> {
    let path = request.path;

    if let Some(index) = path.strip_prefix("/fragments/project/") {
        return fragments::project_detail(index.parse().ok()?).map(fragment);
    }

    let rest = path.strip_prefix("/blog/")?;
    if let Some(slug) = rest.strip_suffix(".md") {
        return content::post(slug).map(|post| markdown(feeds::post_markdown(post)));
    }
    match rest.strip_suffix('/') {
        Some(slug) => {
            let post = content::post(slug)?;
            Some(if request.prefers_markdown() {
                markdown(feeds::post_markdown(post))
            } else {
                html(pages::blog::post(post)).header("Link", PAGE_LINKS)
            })
        }
        // `/blog/slug` → `/blog/slug/`
        None => {
            content::post(rest).map(|post| Response::redirect(&format!("/blog/{}/", post.slug)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get(path: &str) -> Response {
        handle(&Request::get(path))
    }

    fn header<'a>(response: &'a Response, name: &str) -> Option<&'a str> {
        response
            .headers
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.as_str())
    }

    #[test]
    fn home_renders_every_section() {
        let response = get("/");
        assert_eq!(response.status, 200);
        assert_eq!(response.content_type, HTML);
        assert!(response.body.starts_with("<!doctype html>"));
        for id in ["about", "projects", "uses", "contact"] {
            assert!(
                response.body.contains(&format!("id=\"{id}\"")),
                "missing #{id}"
            );
        }
        assert!(response.body.contains("data-distro=\"debian\""));
        assert!(response.body.contains("/vendor/htmx.min.js"));
        assert!(response.body.contains("/vendor/hyperscript.min.js"));
        assert!(header(&response, "Link").unwrap().contains("llms.txt"));
    }

    #[test]
    fn home_negotiates_markdown() {
        let request = Request {
            method: "GET",
            path: "/",
            query: "",
            accept: "text/markdown",
        };
        let response = handle(&request);
        assert_eq!(response.content_type, MARKDOWN);
        assert!(response.body.starts_with("# "));
        assert_eq!(get("/index.md").body, response.body);
    }

    #[test]
    fn blog_index_and_posts() {
        let index = get("/blog/");
        assert_eq!(index.status, 200);
        let post = &content::posts()[0];
        assert!(index.body.contains(&format!("/blog/{}/", post.slug)));

        let page = get(&format!("/blog/{}/", post.slug));
        assert_eq!(page.status, 200);
        assert!(page.body.contains(&post.title));
        assert!(page.body.contains(&format!("/blog/{}.md", post.slug)));

        let twin = get(&format!("/blog/{}.md", post.slug));
        assert_eq!(twin.content_type, MARKDOWN);
        assert!(twin.body.contains(&post.title));

        let redirect = get(&format!("/blog/{}", post.slug));
        assert_eq!(redirect.status, 302);
        assert_eq!(
            header(&redirect, "Location"),
            Some(format!("/blog/{}/", post.slug).as_str())
        );
    }

    #[test]
    fn fragments_for_htmx() {
        let detail = get("/fragments/project/0");
        assert_eq!(detail.status, 200);
        assert!(!detail.body.contains("<html"));
        assert_eq!(get("/fragments/project/9999").status, 404);
        assert_eq!(get("/fragments/project/abc").status, 404);

        let request = Request {
            method: "GET",
            path: "/fragments/posts",
            query: "q=markdown",
            accept: "",
        };
        let filtered = handle(&request);
        assert!(filtered.body.contains("markdown-style-guide"));
        assert!(!filtered.body.contains("first-post"));
    }

    #[test]
    fn feeds_are_well_formed() {
        let rss = get("/rss.xml");
        assert_eq!(rss.content_type, RSS);
        assert_eq!(rss.body.matches("<item>").count(), content::posts().len());
        assert!(get("/sitemap-index.xml").body.contains("/sitemap-0.xml"));
        assert!(get("/sitemap-0.xml").body.contains("/blog/"));
        assert!(get("/llms.txt").body.contains("## Blog"));

        let data: serde_json::Value = serde_json::from_str(&get("/api/data.json").body).unwrap();
        assert_eq!(
            data["posts"].as_array().unwrap().len(),
            content::posts().len()
        );
        assert_eq!(
            data["systems"].as_array().unwrap().len(),
            config::config().systems.len()
        );
    }

    #[test]
    fn lab_page_loads_both_runtimes() {
        let lab = get("/lab/");
        assert_eq!(lab.status, 200);
        assert!(lab.body.contains("/vendor/fengari-web.js"));
        assert!(lab.body.contains("/js/lab.js"));
    }

    #[test]
    fn unknown_paths_and_methods() {
        let missing = get("/nope/<b>");
        assert_eq!(missing.status, 404);
        assert!(!missing.body.contains("/nope/<b>"), "path must be escaped");
        assert_eq!(get("/blog/no-such-post/").status, 404);

        let post = Request {
            method: "POST",
            path: "/",
            query: "",
            accept: "",
        };
        assert_eq!(handle(&post).status, 405);
    }
}
