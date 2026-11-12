//! Minimal request and response types, so the same router runs on Cloudflare
//! Workers (wasm), on the native server and inside the Tauri shell.

#[derive(Debug, Clone, Copy)]
pub struct Request<'a> {
    pub method: &'a str,
    /// Path only, without the query string.
    pub path: &'a str,
    /// Raw query string without the leading `?`.
    pub query: &'a str,
    /// The `Accept` header, or an empty string.
    pub accept: &'a str,
}

impl<'a> Request<'a> {
    pub fn get(path: &'a str) -> Self {
        Self {
            method: "GET",
            path,
            query: "",
            accept: "",
        }
    }

    /// First value of a query parameter, percent-decoded.
    pub fn param(&self, name: &str) -> Option<String> {
        self.query.split('&').find_map(|pair| {
            let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
            (key == name).then(|| percent_decode(value))
        })
    }

    /// Whether the client asks for Markdown ahead of HTML.
    pub fn prefers_markdown(&self) -> bool {
        let position = |needle: &str| self.accept.find(needle);
        match (position("text/markdown"), position("text/html")) {
            (Some(markdown), Some(html)) => markdown < html,
            (Some(_), None) => true,
            _ => false,
        }
    }
}

fn percent_decode(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'+' => out.push(b' '),
            b'%' if i + 2 < bytes.len() => {
                let hex = std::str::from_utf8(&bytes[i + 1..i + 3]).ok();
                match hex.and_then(|h| u8::from_str_radix(h, 16).ok()) {
                    Some(byte) => {
                        out.push(byte);
                        i += 2;
                    }
                    None => out.push(b'%'),
                }
            }
            byte => out.push(byte),
        }
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

#[derive(Debug, Clone)]
pub struct Response {
    pub status: u16,
    pub content_type: &'static str,
    pub headers: Vec<(&'static str, String)>,
    pub body: String,
}

pub const HTML: &str = "text/html; charset=utf-8";
pub const MARKDOWN: &str = "text/markdown; charset=utf-8";
pub const TEXT: &str = "text/plain; charset=utf-8";
pub const JSON: &str = "application/json; charset=utf-8";
pub const XML: &str = "application/xml; charset=utf-8";
pub const RSS: &str = "application/rss+xml; charset=utf-8";

impl Response {
    pub fn new(status: u16, content_type: &'static str, body: impl Into<String>) -> Self {
        Self {
            status,
            content_type,
            headers: Vec::new(),
            body: body.into(),
        }
    }

    pub fn ok(content_type: &'static str, body: impl Into<String>) -> Self {
        Self::new(200, content_type, body)
    }

    pub fn redirect(location: &str) -> Self {
        Self::new(302, TEXT, "").header("Location", location)
    }

    pub fn header(mut self, name: &'static str, value: impl Into<String>) -> Self {
        self.headers.push((name, value.into()));
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_query_parameters() {
        let request = Request {
            method: "GET",
            path: "/",
            query: "q=hello+w%C3%B6rld&x=1",
            accept: "",
        };
        assert_eq!(request.param("q").as_deref(), Some("hello wörld"));
        assert_eq!(request.param("x").as_deref(), Some("1"));
        assert_eq!(request.param("missing"), None);
        let odd = Request {
            method: "GET",
            path: "/",
            query: "q=100%&r=%zz",
            accept: "",
        };
        assert_eq!(odd.param("q").as_deref(), Some("100%"));
        assert_eq!(odd.param("r").as_deref(), Some("%zz"));
    }

    #[test]
    fn negotiates_markdown() {
        let with = |accept| Request {
            method: "GET",
            path: "/",
            query: "",
            accept,
        };
        assert!(with("text/markdown").prefers_markdown());
        assert!(with("text/markdown, text/html;q=0.5").prefers_markdown());
        assert!(!with("text/html, text/markdown").prefers_markdown());
        assert!(!with("text/html,application/xhtml+xml,*/*;q=0.8").prefers_markdown());
        assert!(!with("").prefers_markdown());
    }
}
