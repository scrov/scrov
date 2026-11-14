use super::tui::{Content, Panel, Prompt};
use crate::config::config;
use leptos::prelude::*;

/// `about` section, styled after lazygit.
#[component]
pub fn Lazygit() -> impl IntoView {
    let about = &config().sections.about;
    let status = &about.status;

    let mut sync = Vec::new();
    if status.ahead > 0 {
        sync.push(format!("↑{} ahead", status.ahead));
    }
    if status.behind > 0 {
        sync.push(format!("↓{} behind", status.behind));
    }
    let sync = (!sync.is_empty()).then(|| sync.join(" · "));

    view! {
        <Prompt cmd=about.command.clone() />
        <div class="grid grid-cols-1 gap-5">
            <Panel label="status".to_string()>
                <Content>
                    <p class="w-full flex items-center flex-wrap gap-0">
                        <span class="font-medium text-accent mr-3">"⎇ " {status.branch.clone()}</span>
                        {format!("· since {} ·", status.since)}
                        {sync
                            .map(|bits| {
                                view! {
                                    <span class="text-fg ml-3">{bits}</span>
                                    <span class="text-border">" · "</span>
                                }
                            })}
                        <span class="text-dim ml-1">
                            {format!("+{} modified · {} untracked", status.modified, status.untracked)}
                        </span>
                    </p>
                </Content>
            </Panel>

            <Panel label="bio (modified)".to_string()>
                <Content>
                    {about
                        .bio
                        .iter()
                        .map(|item| {
                            let untracked = item.icon == "??";
                            view! {
                                <p class="w-full flex items-center gap-3">
                                    <span class=if untracked {
                                        "font-medium text-dim"
                                    } else {
                                        "font-medium text-accent"
                                    }>{item.icon.clone()}</span>
                                    <span class=if untracked { "" } else { "ml-2" }>{item.text.clone()}</span>
                                </p>
                            }
                        })
                        .collect_view()}
                </Content>
            </Panel>

            <Panel label="paths (local)".to_string()>
                <Content>
                    {about
                        .paths
                        .iter()
                        .map(|path| {
                            if path.current {
                                view! {
                                    <p class="w-full flex items-center gap-3">
                                        <span class="font-medium text-accent">"* " {path.name.clone()}</span>
                                    </p>
                                }
                                    .into_any()
                            } else {
                                view! {
                                    <p class="w-full flex items-center gap-3 ml-3.5">{path.name.clone()}</p>
                                }
                                    .into_any()
                            }
                        })
                        .collect_view()}
                </Content>
            </Panel>
        </div>
    }
}
