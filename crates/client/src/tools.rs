//! The tools the site offers to an agent driving the browser.
//!
//! Each function builds one tool with webmcp-rs and returns its registration.
//! Argument structs double as the JSON Schema the agent plans against
//! (`register_typed`), so the two cannot drift apart.

use crate::{
    bridge,
    page::{filter_posts, is_slug},
};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;
use webmcp::{Tool, ToolHandle, ToolResult};

/// A tool's registration: the handle that keeps it alive, or why it failed.
pub type Registration = webmcp::Result<ToolHandle>;

/// Available on every page.
pub fn site_tools() -> Vec<Registration> {
    vec![
        get_profile(),
        list_projects(),
        list_posts(),
        read_post(),
        list_themes(),
        set_theme(),
    ]
}

/// Only on the blog index, where the post filter is on screen.
pub fn blog_tools() -> Vec<Registration> {
    vec![filter_blog()]
}

/// Only on the lab page, where the Postgres and Lua shells run.
pub fn lab_tools() -> Vec<Registration> {
    vec![run_sql(), run_lua()]
}

fn get_profile() -> Registration {
    Tool::new("get_profile")
        .title("Get profile")
        .description(
            "Returns the scrov site's home page as Markdown: who runs it, their projects, \
             the tools they use and how to contact them. Use it to answer questions about \
             the site's owner.",
        )
        .read_only()
        .register(|_| async {
            let markdown = bridge::fetch_text("/index.md")
                .await?
                .ok_or_else(|| "the profile is not available".to_string())?;
            Ok::<_, String>(ToolResult::text(markdown))
        })
}

fn list_projects() -> Registration {
    Tool::new("list_projects")
        .title("List projects")
        .description(
            "Lists the owner's projects with name, description, repository, stars and \
             when each was last updated.",
        )
        .read_only()
        .register(|_| async {
            let data = bridge::site_data().await?;
            Ok::<_, String>(ToolResult::json(&data.projects))
        })
}

#[derive(Deserialize, JsonSchema)]
struct ListPosts {
    /// Text to look for in post titles and descriptions, ignoring case. Leave
    /// it out to list every post.
    #[serde(default)]
    query: Option<String>,
}

fn list_posts() -> Registration {
    Tool::new("list_posts")
        .title("List blog posts")
        .description(
            "Lists blog posts, newest first, with slug, title, date, description and URL. \
             Pass `query` to keep only posts that mention it. Does not change the page.",
        )
        .read_only()
        .register_typed(|args: ListPosts| async move {
            let data = bridge::site_data().await?;
            let posts = filter_posts(data.posts, args.query.as_deref().unwrap_or(""));
            Ok::<_, String>(ToolResult::json(&posts))
        })
}

#[derive(Deserialize, JsonSchema)]
struct ReadPost {
    /// The post's slug, as returned by `list_posts`.
    slug: String,
}

fn read_post() -> Registration {
    Tool::new("read_post")
        .title("Read a blog post")
        .description(
            "Returns the full text of one blog post as Markdown. Get the slug from \
             `list_posts` first.",
        )
        .read_only()
        .register_typed(|args: ReadPost| async move {
            if !is_slug(&args.slug) {
                return Ok(ToolResult::error(format!(
                    "`{}` is not a post slug; use one returned by list_posts",
                    args.slug
                )));
            }
            let url = format!("/blog/{}.md", args.slug);
            Ok::<_, String>(match bridge::fetch_text(&url).await? {
                Some(markdown) => ToolResult::text(markdown),
                None => {
                    ToolResult::error(format!("there is no post with the slug `{}`", args.slug))
                }
            })
        })
}

fn list_themes() -> Registration {
    Tool::new("list_themes")
        .title("List themes")
        .description(
            "Lists the site's colour themes, one per operating system, and says which one \
             the page is showing now.",
        )
        .read_only()
        .register(|_| async {
            let data = bridge::site_data().await?;
            Ok::<_, String>(ToolResult::json(&json!({
                "current": bridge::current_theme(),
                "themes": data.systems,
            })))
        })
}

#[derive(Deserialize, JsonSchema)]
struct SetTheme {
    /// A theme slug from `list_themes`, for example `debian` or `nixos`.
    theme: String,
}

fn set_theme() -> Registration {
    Tool::new("set_theme")
        .title("Switch theme")
        .description(
            "Switches the site to another colour theme and remembers the choice in this \
             browser. Changes what the user sees. Use a slug from `list_themes`.",
        )
        .register_typed(|args: SetTheme| async move {
            if !is_slug(&args.theme) || !bridge::theme_exists(&args.theme)? {
                return Ok(ToolResult::error(format!(
                    "`{}` is not a theme; use a slug from list_themes",
                    args.theme
                )));
            }
            bridge::set_theme(&args.theme)?;
            Ok::<_, String>(ToolResult::text(format!(
                "The theme is now {}.",
                args.theme
            )))
        })
}

#[derive(Deserialize, JsonSchema)]
struct FilterBlog {
    /// Text to filter the visible posts by. An empty string shows every post.
    query: String,
}

fn filter_blog() -> Registration {
    Tool::new("filter_blog")
        .title("Filter the post list")
        .description(
            "Types into the filter box on the blog page, so the user sees only matching \
             posts, and returns those posts. To read posts without changing the page, use \
             `list_posts`.",
        )
        .register_typed(|args: FilterBlog| async move {
            bridge::type_into_blog_filter(&args.query)?;
            let data = bridge::site_data().await?;
            let posts = filter_posts(data.posts, &args.query);
            Ok::<_, String>(ToolResult::json(&json!({
                "query": args.query.trim(),
                "shown": posts.len(),
                "posts": posts,
            })))
        })
}

#[derive(Deserialize, JsonSchema)]
struct RunSql {
    /// One or more SQL statements for Postgres.
    query: String,
}

fn run_sql() -> Registration {
    Tool::new("run_sql")
        .title("Run SQL in the lab")
        .description(
            "Runs SQL in the Postgres database inside this browser tab (PGlite) and returns \
             the result as a text table; the user sees it in the lab page too. Tables: \
             projects, posts, systems. The database is private to this tab and resets on \
             reload, so writes are safe to try.",
        )
        .register_typed(|args: RunSql| async move {
            let output = bridge::run_in_lab("sql", &args.query).await?;
            Ok::<_, String>(ToolResult::text(output))
        })
}

#[derive(Deserialize, JsonSchema)]
struct RunLua {
    /// Lua 5.3 source. `print` output is returned.
    code: String,
}

fn run_lua() -> Registration {
    Tool::new("run_lua")
        .title("Run Lua in the lab")
        .description(
            "Runs Lua 5.3 code inside this browser tab (Fengari) and returns what it \
             printed; the user sees it in the lab page too. The global table `site` holds \
             `projects`, `posts` and `systems`.",
        )
        .register_typed(|args: RunLua| async move {
            let output = bridge::run_in_lab("lua", &args.code).await?;
            Ok::<_, String>(ToolResult::text(output))
        })
}

#[cfg(all(test, not(target_family = "wasm")))]
mod tests {
    use super::*;

    /// Off the web, webmcp-rs still validates each descriptor and its derived
    /// schema, then refuses to register with `Unsupported`. Anything else
    /// means a tool would be rejected in the browser.
    #[test]
    fn every_descriptor_is_valid() {
        let all: Vec<Registration> = site_tools()
            .into_iter()
            .chain(blog_tools())
            .chain(lab_tools())
            .collect();
        assert_eq!(all.len(), 9);
        for registration in all {
            assert!(
                matches!(registration, Err(webmcp::Error::Unsupported)),
                "unexpected registration result: {registration:?}"
            );
        }
    }
}
