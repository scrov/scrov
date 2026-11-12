//! HTML fragments that htmx swaps into the page, written with shtml.
//!
//! The same functions also render the initial state inside the Leptos pages,
//! so a fragment and its first paint can never drift apart.

#![allow(non_snake_case)]

use crate::{
    config::config,
    content::{self, Post},
};
use shtml::{html, Component, Elements, Render};

fn Panel(label: &str, elements: Elements) -> Component {
    html! {
        <div class="relative border border-dim w-full flex items-center h-full">
            <span class="absolute -top-3 left-3 bg-bg px-2 text-dim">{label}</span>
            <div class="flex w-full flex-col gap-0.5 p-3">{elements}</div>
        </div>
    }
}

fn ExternalLink(href: &str, label: &str) -> Component {
    html! {
        <a class="mt-[0.4rem] mr-4 inline-block text-accent hover:underline" href=href target="_blank" rel="noopener">
            {label}
        </a>
    }
}

/// Detail pane for one project of the `projects` section.
pub fn project_detail(index: usize) -> Option<String> {
    let project = config().sections.projects.projects.get(index)?;
    let description = if project.description.is_empty() {
        "(no description)"
    } else {
        project.description.as_str()
    };
    let meta = format!("{} · {}", project.stars.trim(), project.updated);
    let homepage: Vec<Component> = project
        .homepage
        .iter()
        .filter(|url| !url.is_empty())
        .map(|url| html! { <ExternalLink href=url.as_str() label="→ visit site" /> })
        .collect();

    Some(
        html! {
            <Panel label=project.name.as_str()>
                <div class="mb-2">{description}</div>
                <div class="text-dim">{project.repo.as_str()}</div>
                <div class="text-dim">{meta}</div>
                <div>
                    <ExternalLink href=project.url.as_str() label="→ repo" />
                    {homepage}
                </div>
            </Panel>
        }
        .to_string(),
    )
}

fn PostRow(post: &Post) -> Component {
    let href = format!("/blog/{}/", post.slug);
    html! {
        <li class="grid grid-cols-12 gap-x-4 py-1">
            <time class="col-span-12 sm:col-span-3 text-dim" datetime=post.date.iso()>{post.date.human()}</time>
            <div class="col-span-12 sm:col-span-9">
                <a class="text-c2 hover:text-accent hover:underline" href=href>{post.title.as_str()}</a>
                <p class="text-dim line-clamp-1">{post.description.as_str()}</p>
            </div>
        </li>
    }
}

/// The blog listing, filtered by `query`.
pub fn post_list(query: &str) -> String {
    let posts = content::search_posts(query);
    if posts.is_empty() {
        let message = format!("grep: no posts match \"{}\"", query.trim());
        return html! { <li class="text-c3">{message}</li> }.to_string();
    }
    let rows: Vec<Component> = posts
        .into_iter()
        .map(|post| html! { <PostRow post=post /> })
        .collect();
    html! { <>{rows}</> }.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_project_detail() {
        let html = project_detail(0).expect("first project exists");
        let project = &config().sections.projects.projects[0];
        assert!(html.contains(&project.url));
        assert!(html.contains("→ repo"));
        assert!(project_detail(usize::MAX).is_none());
    }

    #[test]
    fn lists_and_filters_posts() {
        let all = post_list("");
        assert_eq!(all.matches("<li").count(), content::posts().len());
        assert!(post_list("zzz-not-there").contains("no posts match"));
    }

    #[test]
    fn escapes_the_search_query() {
        let html = post_list("<script>alert(1)</script>");
        assert!(!html.contains("<script>"));
    }
}
