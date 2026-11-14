use super::tui::{Content, Panel, Prompt};
use crate::config::config;
use leptos::prelude::*;

/// `uses` section, styled after btop.
#[component]
pub fn Btop() -> impl IntoView {
    let uses = &config().sections.uses;
    let memory = &uses.memory;
    let used_percent = if memory.total > 0.0 {
        memory.used / memory.total * 100.0
    } else {
        0.0
    };

    let peripherals = &uses.peripherals;
    let items: Vec<(&'static str, String)> = [
        ("keyboard", &peripherals.keyboard),
        ("mouse", &peripherals.mouse),
        ("headphones", &peripherals.headphones),
        ("monitor", &peripherals.monitor),
    ]
    .into_iter()
    .filter_map(|(key, value)| value.clone().map(|value| (key, value)))
    .collect();

    view! {
        <Prompt cmd=uses.command.clone() />
        <div class="my-2 flex flex-col gap-5">
            <Panel label="CPU [daily usage]".to_string()>
                <Content>
                    <div class="font-mono grid grid-cols-12 items-center gap-x-3 gap-y-1">
                        {uses
                            .cpu
                            .usage
                            .iter()
                            .map(|usage| {
                                view! {
                                    <span class="col-span-3 sm:col-span-2">{usage.label.clone()}</span>
                                    <span class="col-span-9 sm:col-span-10 md:col-span-5 flex items-center gap-2">
                                        <span class="flex-1 h-4 overflow-hidden bg-accent/10">
                                            <span
                                                class="block h-full bg-accent"
                                                style=format!("width: {}%", usage.percent.min(100))
                                            ></span>
                                        </span>
                                        <span class="text-dim whitespace-pre">{format!("{:>3}%", usage.percent)}</span>
                                    </span>
                                    <span class="col-span-12 ml-2 md:ml-0 md:col-span-5 gap-2">
                                        <span class="line-clamp-1">
                                            {usage.detail.clone()}
                                            {usage
                                                .note
                                                .clone()
                                                .map(|note| view! { <span class="text-dim">" " {note}</span> })}
                                        </span>
                                    </span>
                                }
                            })
                            .collect_view()}
                    </div>
                </Content>
            </Panel>

            <Panel label="MEM".to_string()>
                <Content>
                    <div class="font-mono grid grid-cols-12 items-center gap-x-3 gap-y-1">
                        <span class="col-span-3 sm:col-span-2">"RAM"</span>
                        <span class="col-span-9 sm:col-span-10 md:col-span-5 flex items-center gap-2">
                            <span class="flex-1 h-4 bg-accent/10">
                                <span class="block h-full bg-accent" style=format!("width: {used_percent:.0}%")></span>
                            </span>
                            <span class="text-dim whitespace-pre">{format!("{used_percent:>3.0}%")}</span>
                        </span>
                        <span class="col-span-12 md:ml-0 md:col-span-5 text-accent whitespace-pre flex gap-2">
                            <span class="text-dim whitespace-pre">{format!("{}/{} GB", memory.used, memory.total)}</span>
                            {memory.theme.clone()}
                            <span class="text-dim">{format!(" {}%", memory.theme_percent)}</span>
                        </span>
                    </div>
                </Content>
            </Panel>

            <Panel label="hardware".to_string()>
                <Content>
                    <div class="font-mono flex items-center gap-x-2">
                        <span>{uses.hardware.device.clone()}</span>
                        <span class="text-dim">"·"</span>
                        <span>{uses.hardware.ram.clone()}</span>
                        <span class="text-dim">"·"</span>
                        <span class="text-accent">{uses.hardware.os.clone()}</span>
                    </div>
                </Content>
            </Panel>

            <Panel label=peripherals.label.clone().unwrap_or_else(|| "peripherals".to_string())>
                <Content>
                    <div class="font-mono flex flex-wrap items-center gap-x-2 gap-y-1">
                        {items
                            .into_iter()
                            .enumerate()
                            .map(|(index, (key, value))| {
                                view! {
                                    {(index > 0).then(|| view! { <span class="text-dim">"·"</span> })}
                                    <span>
                                        <span class="text-fg">{key}</span>
                                        ": "
                                        <span class="text-accent">{value}</span>
                                    </span>
                                }
                            })
                            .collect_view()}
                    </div>
                </Content>
            </Panel>
        </div>
    }
}
