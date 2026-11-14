//! The few places the tools touch the page: `fetch`, the theme function from
//! `public/js/site.js`, the blog filter box and the lab shells from
//! `public/js/lab.js`.

use crate::page::SiteData;
use js_sys::{Function, Promise, Reflect};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;

/// Text of a thrown JavaScript value.
fn describe(value: JsValue) -> String {
    if let Some(text) = value.as_string() {
        return text;
    }
    Reflect::get(&value, &JsValue::from_str("message"))
        .ok()
        .and_then(|message| message.as_string())
        .unwrap_or_else(|| format!("{value:?}"))
}

fn window() -> Result<web_sys::Window, String> {
    web_sys::window().ok_or_else(|| "this page has no window".to_string())
}

fn document() -> Result<web_sys::Document, String> {
    window()?
        .document()
        .ok_or_else(|| "this page has no document".to_string())
}

/// The current path, or an empty string outside a browser.
pub fn pathname() -> String {
    web_sys::window()
        .and_then(|window| window.location().pathname().ok())
        .unwrap_or_default()
}

/// GET a same-origin URL and return its body. `Ok(None)` means 404.
pub async fn fetch_text(url: &str) -> Result<Option<String>, String> {
    let response = JsFuture::from(window()?.fetch_with_str(url))
        .await
        .map_err(describe)?;
    let response: web_sys::Response = response
        .dyn_into()
        .map_err(|_| "fetch did not return a Response".to_string())?;
    if response.status() == 404 {
        return Ok(None);
    }
    if !response.ok() {
        return Err(format!("{url} answered HTTP {}", response.status()));
    }
    let text = JsFuture::from(response.text().map_err(describe)?)
        .await
        .map_err(describe)?;
    text.as_string()
        .map(Some)
        .ok_or_else(|| format!("{url} did not return text"))
}

/// The site's projects, posts and systems.
pub async fn site_data() -> Result<SiteData, String> {
    let body = fetch_text("/api/data.json")
        .await?
        .ok_or_else(|| "/api/data.json is missing".to_string())?;
    serde_json::from_str(&body).map_err(|error| format!("/api/data.json: {error}"))
}

/// The theme the page is showing now.
pub fn current_theme() -> Option<String> {
    document()
        .ok()?
        .document_element()?
        .get_attribute("data-theme")
}

/// Whether the theme menu offers `theme`. The caller validates the slug first.
pub fn theme_exists(theme: &str) -> Result<bool, String> {
    let selector = format!("[data-theme-option=\"{theme}\"]");
    document()?
        .query_selector(&selector)
        .map(|found| found.is_some())
        .map_err(describe)
}

/// Apply a theme through `scrovTheme` from `site.js`, which also stores it.
pub fn set_theme(theme: &str) -> Result<(), String> {
    let window = window()?;
    let apply: Function = Reflect::get(&window, &JsValue::from_str("scrovTheme"))
        .ok()
        .and_then(|value| value.dyn_into().ok())
        .ok_or_else(|| "the page's theme switcher has not loaded".to_string())?;
    apply
        .call1(&window, &JsValue::from_str(theme))
        .map(|_| ())
        .map_err(describe)
}

/// Type `query` into the blog filter box. htmx listens for the `input` event
/// and swaps in the matching posts.
pub fn type_into_blog_filter(query: &str) -> Result<(), String> {
    let input: web_sys::HtmlInputElement = document()?
        .query_selector("input[name=\"q\"]")
        .map_err(describe)?
        .and_then(|element| element.dyn_into().ok())
        .ok_or_else(|| "this page has no post filter".to_string())?;
    input.set_value(query);

    let init = web_sys::EventInit::new();
    init.set_bubbles(true);
    let event = web_sys::Event::new_with_event_init_dict("input", &init).map_err(describe)?;
    input.dispatch_event(&event).map(|_| ()).map_err(describe)
}

/// Run `source` in one of the lab's shells (`sql` or `lua`) and return its
/// output. `public/js/lab.js` publishes both on `window.scrovLab`; each waits
/// for its runtime to finish loading and shows the run in the page.
pub async fn run_in_lab(shell: &str, source: &str) -> Result<String, String> {
    let window: JsValue = window()?.into();
    let lab = Reflect::get(&window, &JsValue::from_str("scrovLab")).map_err(describe)?;
    if !lab.is_object() {
        return Err("the lab has not loaded yet; try again in a moment".to_string());
    }
    let run: Function = Reflect::get(&lab, &JsValue::from_str(shell))
        .ok()
        .and_then(|value| value.dyn_into().ok())
        .ok_or_else(|| format!("the lab has no {shell} shell"))?;

    let returned = run
        .call1(&lab, &JsValue::from_str(source))
        .map_err(describe)?;
    let output = match returned.dyn_into::<Promise>() {
        Ok(promise) => JsFuture::from(promise).await.map_err(describe)?,
        Err(value) => value,
    };
    output
        .as_string()
        .ok_or_else(|| format!("the {shell} shell did not return text"))
}
