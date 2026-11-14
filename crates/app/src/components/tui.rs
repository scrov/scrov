//! Shared pieces of the terminal-UI panels.

use leptos::prelude::*;

/// Bordered panel with a label sitting on its top edge.
#[component]
pub fn Panel(label: String, children: Children) -> impl IntoView {
    view! {
        <div class="relative border border-dim w-full flex items-center h-full">
            <span class="absolute -top-3 left-3 bg-bg px-2 text-dim">{label}</span>
            {children()}
        </div>
    }
}

/// Padded column inside a panel.
#[component]
pub fn Content(children: Children) -> impl IntoView {
    view! { <div class="flex w-full flex-col gap-0.5 p-3">{children()}</div> }
}

/// `$ command` line above a section.
#[component]
pub fn Prompt(cmd: String, #[prop(default = "mb-5")] class: &'static str) -> impl IntoView {
    view! {
        <h3 class=format!("text-dim {class}")>
            <span class="text-accent">"$"</span>
            " "
            {cmd}
        </h3>
    }
}

#[component]
pub fn SectionTitle(title: &'static str) -> impl IntoView {
    view! { <h2 class="mb-4 font-medium tracking-wider text-accent">"~ " {title}</h2> }
}
