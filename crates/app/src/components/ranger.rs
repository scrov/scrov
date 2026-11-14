use super::tui::{Panel, Prompt};
use crate::{config::config, fragments};
use leptos::prelude::*;

/// `projects` section, styled after ranger. Picking a project swaps the
/// detail pane with an htmx request to `/fragments/project/{index}`.
#[component]
pub fn Ranger() -> impl IntoView {
    let section = &config().sections.projects;
    let count = section.projects.len();
    let first_detail = fragments::project_detail(0).unwrap_or_default();

    view! {
        <Prompt cmd=section.command.clone() />
        <div class="my-2 grid grid-cols-1">
            <Panel label="tree".to_string()>
                <pre class="m-0 overflow-x-auto p-4 flex flex-col w-full">
                    <span class="text-dim">{section.root.clone()}</span>
                    <span data-tree class="inline-flex gap-2">
                        "projects/"
                        <span class="text-dim">{format!("({count} items)")}</span>
                    </span>
                    {section
                        .projects
                        .iter()
                        .enumerate()
                        .map(|(index, project)| {
                            view! {
                                <div
                                    data-tree-row
                                    class="flex gap-2 ml-8"
                                    data-last=if index + 1 == count { "true" } else { "false" }
                                >
                                    <button
                                        type="button"
                                        data-tree-item=index.to_string()
                                        class=if index == 0 {
                                            "cursor-pointer hover:text-accent text-accent"
                                        } else {
                                            "cursor-pointer hover:text-accent"
                                        }
                                        hx-get=format!("/fragments/project/{index}")
                                        hx-target="#project-detail"
                                        hx-swap="innerHTML"
                                        data-script="on click remove .text-accent from <button[data-tree-item]/> then add .text-accent to me"
                                    >
                                        {project.name.clone()}
                                    </button>
                                    <div class="flex gap-2 w-full justify-end">
                                        <span class="text-dim">{project.size.clone()}</span>
                                        <span class="text-dim">{project.date.clone()}</span>
                                    </div>
                                </div>
                            }
                        })
                        .collect_view()}
                </pre>
            </Panel>
            <div id="project-detail" class="h-full" inner_html=first_detail></div>
        </div>
    }
}
