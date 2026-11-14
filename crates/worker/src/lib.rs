//! Cloudflare Workers entry point.
//!
//! Static files in `public/` are served by Workers Static Assets before this
//! code runs; every other request is rendered by [`scrov_app::handle`].

use worker::{event, Context, Env, Headers, Request, Response, Result};

#[event(fetch)]
async fn fetch(request: Request, _env: Env, _ctx: Context) -> Result<Response> {
    let url = request.url()?;
    let accept = request.headers().get("Accept")?.unwrap_or_default();
    let method = request.method().to_string();

    let page = scrov_app::handle(&scrov_app::Request {
        method: &method,
        path: url.path(),
        query: url.query().unwrap_or(""),
        accept: &accept,
    });

    let headers = Headers::new();
    headers.set("Content-Type", page.content_type)?;
    for (name, value) in &page.headers {
        headers.set(name, value)?;
    }
    // A HEAD response carries the headers of the GET response, without a body.
    let body = if method == "HEAD" {
        String::new()
    } else {
        page.body
    };
    Ok(Response::ok(body)?
        .with_status(page.status)
        .with_headers(headers))
}
