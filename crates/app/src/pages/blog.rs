use crate::{
    components::tui::{Prompt, SectionTitle},
    content::Post,
    fragments,
    layout::{page, Meta},
};
use leptos::prelude::*;

/// Blog index with an htmx-powered filter.
pub fn index() -> String {
    let mut meta = Meta::new("blog", "/blog/");
    meta.description = Some("Posts from the scrov blog.");

    page(&meta, || {
        view! {
            <section class="my-6">
                <SectionTitle title="blog" />
                <Prompt cmd="ls -lt ~/blog | grep -i".to_string() class="mb-2" />
                <input
                    type="search"
                    name="q"
                    autocomplete="off"
                    placeholder="pattern"
                    aria-label="Filter posts"
                    class="mb-5 w-full border border-dim bg-panel px-3 py-1 text-fg placeholder:text-dim focus:border-accent focus:outline-none"
                    hx-get="/fragments/posts"
                    hx-trigger="input changed delay:200ms, search"
                    hx-target="#posts"
                    hx-swap="innerHTML"
                />
                <ul id="posts" class="font-mono" inner_html=fragments::post_list("")></ul>
            </section>
        }
    })
}

/// One blog post.
pub fn post(post: &'static Post) -> String {
    let path = format!("/blog/{}/", post.slug);
    let mut meta = Meta::new(&post.title, &path);
    meta.description = Some(&post.description);
    meta.og_type = "article";
    meta.markdown = Some(format!("/blog/{}.md", post.slug));

    page(&meta, || {
        view! {
            <article class="my-6">
                <Prompt cmd=format!("glow ~/blog/{}.md", post.slug) class="mb-5" />
                {post
                    .hero_image
                    .clone()
                    .map(|src| {
                        view! { <img class="mb-6 w-full border border-dim" src=src alt="" width="1020" height="510" /> }
                    })}
                <h1 class="text-accent font-medium">{post.title.clone()}</h1>
                <p class="mb-6 text-dim">
                    <time datetime=post.date.iso()>{post.date.human()}</time>
                </p>
                <div class="prose" inner_html=post.html.clone()></div>
                <p class="mt-10">
                    <span class="text-accent">"$ "</span>
                    <a href="/blog/" class="text-c2 hover:text-accent hover:underline">"cd .."</a>
                </p>
            </article>
        }
    })
}
