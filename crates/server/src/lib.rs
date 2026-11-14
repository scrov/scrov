//! Serves the scrov site over HTTP with axum.
//!
//! Static files come from the `public/` directory; everything else goes to
//! [`scrov_app::handle`], the same router the Cloudflare Worker runs.

use axum::{
    body::Body,
    http::{header, HeaderName, HeaderValue, Request, StatusCode},
    response::Response,
    Router,
};
use std::path::PathBuf;
use tower_http::{compression::CompressionLayer, services::ServeDir};

/// Directory with the static assets: `$SCROV_PUBLIC_DIR`, or `./public`.
pub fn public_dir() -> PathBuf {
    std::env::var_os("SCROV_PUBLIC_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("public"))
}

async fn render(request: Request<Body>) -> Response {
    let accept = request
        .headers()
        .get(header::ACCEPT)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("");
    let page = scrov_app::handle(&scrov_app::Request {
        method: request.method().as_str(),
        path: request.uri().path(),
        query: request.uri().query().unwrap_or(""),
        accept,
    });

    let mut response = Response::new(Body::from(page.body));
    *response.status_mut() =
        StatusCode::from_u16(page.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    let headers = response.headers_mut();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static(page.content_type),
    );
    for (name, value) in page.headers {
        if let (Ok(name), Ok(value)) = (
            HeaderName::from_bytes(name.as_bytes()),
            HeaderValue::from_str(&value),
        ) {
            headers.insert(name, value);
        }
    }
    response
}

/// The whole site as an axum router: static files first, then rendered pages.
pub fn router(public: impl Into<PathBuf>) -> Router {
    // Static directories with an index.html (/platform/, /lab/intent/) are
    // served as pages, as Workers Static Assets does. A directory without one
    // (/lab/) falls through to the router.
    let files = ServeDir::new(public.into())
        .append_index_html_on_directories(true)
        .fallback(tower::service_fn(|request: Request<Body>| async move {
            Ok::<_, std::convert::Infallible>(render(request).await)
        }));
    Router::new()
        .fallback_service(files)
        .layer(CompressionLayer::new())
}

/// Serves the site on `listener` until the task is dropped.
pub async fn serve(listener: tokio::net::TcpListener, public: PathBuf) -> std::io::Result<()> {
    axum::serve(listener, router(public)).await
}
