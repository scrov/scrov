//! Browser side of the scrov site.
//!
//! The pages themselves are rendered on the server and never hydrated. This
//! crate is the one piece of Leptos that runs in the browser: a small island
//! mounted into the footer that registers [WebMCP] tools with [webmcp-rs], so
//! an agent driving the browser can call the site instead of scraping it.
//!
//! Tools are registered from component bodies with `webmcp::leptos::use_tool`,
//! which parks each handle in the component's reactive owner. A tool therefore
//! exists exactly as long as the component that offers it: the blog filter
//! tool only on the blog index, the SQL and Lua tools only on the lab page.
//!
//! [WebMCP]: https://github.com/webmachinelearning/webmcp
//! [webmcp-rs]: https://github.com/hauju/webmcp-rs

pub mod bridge;
pub mod page;
pub mod tools;

use leptos::prelude::*;
use page::Page;
use tools::Registration;

/// Hand each registration to the calling component's owner and count the ones
/// the browser accepted.
fn offer(tools: Vec<Registration>, registered: RwSignal<usize>) {
    for tool in tools {
        if tool.is_ok() {
            registered.update(|count| *count += 1);
        }
        webmcp::leptos::use_tool(move || tool);
    }
}

#[component]
fn SiteTools(registered: RwSignal<usize>) -> impl IntoView {
    offer(tools::site_tools(), registered);
}

#[component]
fn BlogTools(registered: RwSignal<usize>) -> impl IntoView {
    offer(tools::blog_tools(), registered);
}

#[component]
fn LabTools(registered: RwSignal<usize>) -> impl IntoView {
    offer(tools::lab_tools(), registered);
}

/// The island: registers the tools for this page and shows how many an agent
/// can see.
#[component]
pub fn AgentTools() -> impl IntoView {
    let registered = RwSignal::new(0usize);
    let page = Page::from_path(&bridge::pathname());
    let supported = webmcp::is_supported();

    let title = if supported {
        "WebMCP tools this page offers to an agent in your browser"
    } else {
        "This browser has no document.modelContext, so no WebMCP tools are registered"
    };

    view! {
        <SiteTools registered />
        {(page == Page::BlogIndex).then(|| view! { <BlogTools registered /> })}
        {(page == Page::Lab).then(|| view! { <LabTools registered /> })}
        <span data-webmcp=if supported { "on" } else { "off" } title=title>
            <span class=if supported { "text-accent" } else { "text-border" }>"◆ "</span>
            {move || {
                if supported {
                    format!("webmcp: {} tools", registered.get())
                } else {
                    "webmcp: off".to_string()
                }
            }}
        </span>
    }
}

/// Mounts the island into `#webmcp-root`, which the server renders in the
/// footer. Pages without that element are left alone.
#[cfg(target_family = "wasm")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() {
    use wasm_bindgen::JsCast;

    console_error_panic_hook::set_once();
    let root = web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.get_element_by_id("webmcp-root"))
        .and_then(|element| element.dyn_into::<web_sys::HtmlElement>().ok());
    if let Some(root) = root {
        leptos::mount::mount_to(root, AgentTools).forget();
    }
}
