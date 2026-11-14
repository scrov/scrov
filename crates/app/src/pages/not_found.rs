use crate::layout::{page, Meta};
use leptos::prelude::*;

/// 404 page, in the voice of a shell.
pub fn render(path: &str) -> String {
    let meta = Meta::new("404 — bash: command not found", "/404");
    let path = path.to_string();

    page(&meta, || {
        view! {
            <div class="font-mono my-6 leading-relaxed">
                <p>
                    <span class="text-accent">"$ "</span>
                    "cd "
                    <span class="text-c2">{path.clone()}</span>
                </p>
                <p class="text-c3">{format!("bash: cd: {path}: No such file or directory")}</p>
                <p>"\u{a0}"</p>
                <p>
                    <span class="text-accent">"$ "</span>
                    <a href="/" class="text-c2 hover:text-accent hover:underline">"cd ~"</a>
                </p>
                <p class="text-dim">"→ back to home"</p>
                <p>"\u{a0}"</p>
                <p>
                    <span class="text-accent">"$ "</span>
                    <span class="text-border">"_"</span>
                </p>
            </div>
        }
    })
}
