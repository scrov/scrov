use crate::{
    components::tui::{Content, Panel, Prompt, SectionTitle},
    layout::{page, Meta},
};
use leptos::prelude::*;

const INPUT: &str = "w-full border border-dim bg-panel px-3 py-2 font-mono text-fg placeholder:text-dim focus:border-accent focus:outline-none";
const BUTTON: &str = "cursor-pointer border border-border bg-panel px-3 py-1 text-fg hover:border-accent hover:text-accent disabled:cursor-wait disabled:opacity-50";
const OUTPUT: &str = "min-h-24 max-h-96 w-full overflow-auto whitespace-pre p-3 text-[0.8rem]";

/// In-browser playground: Postgres (PGlite) and Lua (Fengari) over the site's data.
pub fn render() -> String {
    let mut meta = Meta::new("lab", "/lab/");
    meta.description = Some(
        "Query the site's data with Postgres and script it with Lua, both running in your browser.",
    );

    page(&meta, || {
        view! {
            <section class="my-6">
                <SectionTitle title="lab" />
                <p class="mb-8 text-dim">
                    "Both shells run entirely in this tab. Nothing you type leaves your browser."
                </p>

                <Prompt cmd="psql scrov".to_string() />
                <form id="sql-form" class="mb-3 flex flex-col gap-2">
                    <textarea id="sql-input" name="sql" rows="3" spellcheck="false" aria-label="SQL" class=INPUT>
                        "select name, stars, updated from projects order by name;"
                    </textarea>
                    <div class="flex flex-wrap items-center gap-3">
                        <button id="sql-run" type="submit" class=BUTTON disabled>"run"</button>
                        <span id="sql-status" class="text-dim">"loading PGlite…"</span>
                    </div>
                </form>
                <Panel label="result".to_string()>
                    <Content>
                        <pre id="sql-output" class=OUTPUT>"\\d  →  tables: projects, posts, systems"</pre>
                    </Content>
                </Panel>

                <div class="h-10"></div>

                <Prompt cmd="lua -i".to_string() />
                <form id="lua-form" class="mb-3 flex flex-col gap-2">
                    <textarea id="lua-input" name="lua" rows="5" spellcheck="false" aria-label="Lua" class=INPUT>
                        "for i, post in ipairs(site.posts) do\n  print(i, post.date, post.title)\nend"
                    </textarea>
                    <div class="flex flex-wrap items-center gap-3">
                        <button id="lua-run" type="submit" class=BUTTON disabled>"run"</button>
                        <span id="lua-status" class="text-dim">"loading Fengari…"</span>
                    </div>
                </form>
                <Panel label="output".to_string()>
                    <Content>
                        <pre id="lua-output" class=OUTPUT>"site.projects, site.posts and site.systems are Lua tables"</pre>
                    </Content>
                </Panel>
            </section>
            <script src="/vendor/fengari-web.js"></script>
            <script type="module" src="/js/lab.js"></script>
        }
    })
}
