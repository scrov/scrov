use crate::{
    components::{
        btop::Btop, lazygit::Lazygit, logo::DistroLogo, mutt::Mutt, ranger::Ranger,
        tui::SectionTitle,
    },
    config::{config, System},
    layout::{page, Meta},
};
use leptos::prelude::*;
use std::sync::LazyLock;

enum Value {
    Fixed(String),
    PerDistro(fn(&System) -> String),
}

fn info_rows() -> Vec<(&'static str, Value)> {
    let current = &config().current;
    vec![
        ("OS", Value::PerDistro(|s| s.os.clone())),
        ("Host", Value::Fixed(current.host.clone())),
        ("Kernel", Value::PerDistro(|s| s.kernel.clone())),
        (
            "Packages",
            Value::PerDistro(|s| format!("{} ({})", s.pkgs, s.pkg)),
        ),
        ("Shell", Value::PerDistro(|s| s.shell.clone())),
        ("Resolution", Value::Fixed(current.resolution.clone())),
        ("DE", Value::PerDistro(|s| s.de.clone())),
        ("WM", Value::PerDistro(|s| s.wm.clone())),
        ("Theme", Value::PerDistro(|s| s.theme.clone())),
        ("Terminal", Value::PerDistro(|s| s.terminal.clone())),
        ("Font", Value::PerDistro(|s| s.font.clone())),
        ("CPU", Value::Fixed(current.cpu.clone())),
        ("GPU", Value::Fixed(current.gpu.clone())),
        ("Memory", Value::Fixed(current.memory.clone())),
    ]
}

const ROW: &str = "mb-[0.05rem] flex items-center gap-2";
const KEY: &str = "inline-block font-medium text-accent min-w-22.5";

/// The fastfetch-style home page.
///
/// It is the largest page (one logo and one set of values per distro) and
/// depends on nothing in the request, so it is rendered once per process.
pub fn render() -> &'static str {
    static PAGE: LazyLock<String> = LazyLock::new(render_page);
    &PAGE
}

fn render_page() -> String {
    let site = &config().site;
    let mut meta = Meta::new(&site.title, "/");
    meta.markdown = Some("/index.md".into());

    page(&meta, || {
        view! {
            <div class="mb-10 flex flex-col items-start md:flex-row gap-6 md:gap-12">
                <DistroLogo />
                <div>
                    <div class="font-medium text-accent mb-1.5">
                        <h1 class="inline border-b border-dim pb-0.5">
                            {site.author.clone()}
                            {(!site.handle_suffix.is_empty())
                                .then(|| {
                                    view! {
                                        <span class="text-dim">"@"</span>
                                        {site.handle_suffix.clone()}
                                    }
                                })}
                        </h1>
                    </div>
                    {info_rows()
                        .into_iter()
                        .map(|(key, value)| {
                            let value = match value {
                                Value::Fixed(text) => view! { <span>{text}</span> }.into_any(),
                                Value::PerDistro(get) => {
                                    config()
                                        .systems
                                        .iter()
                                        .map(|system| {
                                            view! { <span data-distro=system.slug.clone()>{get(system)}</span> }
                                        })
                                        .collect_view()
                                        .into_any()
                                }
                            };
                            view! {
                                <div class=ROW>
                                    <span class=KEY>{key}</span>
                                    {value}
                                </div>
                            }
                        })
                        .collect_view()}
                    <div class=ROW>
                        <span class=KEY>"Uptime"</span>
                        <span data-uptime=config().current.booted_at.clone() class="text-dim">"—"</span>
                    </div>
                </div>
            </div>

            <section id="about" class="my-10">
                <SectionTitle title="about" />
                <Lazygit />
            </section>
            <section id="projects" class="my-10">
                <SectionTitle title="projects" />
                <Ranger />
            </section>
            <section id="uses" class="my-10">
                <SectionTitle title="uses" />
                <Btop />
            </section>
            <section id="contact" class="my-10">
                <SectionTitle title="contact" />
                <Mutt />
            </section>
        }
    })
}
