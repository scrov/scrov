use crate::{config::config, content};
use leptos::prelude::*;

/// One ASCII logo per distro; CSS shows the one matching `data-theme`.
#[component]
pub fn DistroLogo() -> impl IntoView {
    config()
        .systems
        .iter()
        .filter_map(|system| {
            let logo = content::logo(&system.slug)?;
            let html = format!(
                "<span class=\"text-2xs\">{}</span>",
                content::logo_html(logo)
            );
            Some(view! {
                <pre
                    data-distro=system.slug.clone()
                    class="leading-[0.82] md:leading-[0.75] whitespace-pre"
                    inner_html=html
                ></pre>
            })
        })
        .collect_view()
}
