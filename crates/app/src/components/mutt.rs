use super::tui::Prompt;
use crate::config::config;
use leptos::prelude::*;

/// `contact` section, styled after mutt.
#[component]
pub fn Mutt() -> impl IntoView {
    let contact = &config().sections.contact;

    view! {
        <Prompt cmd=contact.command.clone() class="mb-[0.6rem]" />
        <div class="font-mono my-2">
            {contact
                .inbox
                .iter()
                .map(|mail| {
                    let external = !mail.href.starts_with("mailto:");
                    view! {
                        <a
                            class="grid grid-cols-12 gap-x-4 group"
                            href=mail.href.clone()
                            target=external.then_some("_blank")
                            rel=external.then_some("noopener")
                        >
                            <span class="col-span-6 sm:col-span-4 hover:cursor-pointer group-hover:text-accent text-c2">
                                {mail.from.clone()}
                            </span>
                            <span class="col-span-6 sm:col-span-8 flex justify-end text-dim">{mail.date.clone()}</span>
                        </a>
                    }
                })
                .collect_view()}
        </div>
        <div class="mt-2 text-[0.8rem] text-dim">
            {contact
                .hints
                .iter()
                .enumerate()
                .map(|(index, hint)| {
                    view! {
                        <span>
                            {(index > 0).then_some(" · ")}
                            <span class="text-accent">{hint.key.clone()}</span>
                            " "
                            {hint.action.clone()}
                        </span>
                    }
                })
                .collect_view()}
        </div>
    }
}
