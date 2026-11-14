use crate::config::{config, themes};
use leptos::prelude::*;

/// Theme menu. Opening and closing is hyperscript; `scrovTheme` (site.js)
/// applies and remembers the choice.
#[component]
pub fn ThemeToggle() -> impl IntoView {
    let default_theme = config().site.default_theme.clone();
    view! {
        <div class="relative inline-block">
            <button
                id="theme-trigger"
                type="button"
                aria-haspopup="listbox"
                class="cursor-pointer border border-border bg-panel px-2 py-1 text-fg hover:border-accent hover:text-accent"
                data-script="on click toggle @hidden on #theme-menu then halt the event"
            >
                <span class="text-dim">"theme: "</span>
                <span data-current-theme>{default_theme}</span>
                <span class="text-border">" ▾"</span>
            </button>
            <ul
                id="theme-menu"
                hidden
                role="listbox"
                class="absolute right-0 bottom-full z-50 mb-1 max-h-64 w-44 overflow-y-auto border border-border bg-panel text-[0.75rem] shadow-lg"
                data-script="on click from elsewhere add @hidden to me end on keyup[key is 'Escape'] from window add @hidden to me"
            >
                {themes()
                    .into_iter()
                    .map(|theme| {
                        view! {
                            <li>
                                <button
                                    type="button"
                                    data-theme-option=theme
                                    role="option"
                                    aria-selected="false"
                                    class="block w-full cursor-pointer px-3 py-1 text-left text-dim hover:bg-fg/10 hover:text-accent aria-selected:text-accent"
                                    data-script="on click call scrovTheme(@data-theme-option) then add @hidden to #theme-menu"
                                >
                                    {theme}
                                </button>
                            </li>
                        }
                    })
                    .collect_view()}
            </ul>
        </div>
    }
}
