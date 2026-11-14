//! Which page the island is running on, and the pure helpers the tools share.

use serde::{Deserialize, Serialize};

/// The kinds of page that register their own tools.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    BlogIndex,
    Lab,
    Other,
}

impl Page {
    pub fn from_path(path: &str) -> Self {
        match path.trim_end_matches('/') {
            "/blog" => Self::BlogIndex,
            "/lab" => Self::Lab,
            _ => Self::Other,
        }
    }
}

/// The JSON served at `/api/data.json`.
#[derive(Debug, Clone, Deserialize)]
pub struct SiteData {
    pub projects: Vec<Project>,
    pub posts: Vec<Post>,
    pub systems: Vec<System>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    pub description: String,
    pub repo: String,
    pub stars: String,
    pub updated: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Post {
    pub slug: String,
    pub title: String,
    pub description: String,
    pub date: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct System {
    pub slug: String,
    pub os: String,
    pub accent: String,
}

/// Posts whose title or description contains `query`, ignoring case. An empty
/// query keeps every post. Mirrors the server's `/fragments/posts` filter.
pub fn filter_posts(posts: Vec<Post>, query: &str) -> Vec<Post> {
    let query = query.trim().to_lowercase();
    posts
        .into_iter()
        .filter(|post| {
            query.is_empty()
                || post.title.to_lowercase().contains(&query)
                || post.description.to_lowercase().contains(&query)
        })
        .collect()
}

/// Whether `value` is safe to place in a URL path segment or a CSS attribute
/// selector: 1 to 64 ASCII letters, digits, `-` or `_`.
pub fn is_slug(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_'))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn post(title: &str, description: &str) -> Post {
        Post {
            slug: title.to_lowercase().replace(' ', "-"),
            title: title.into(),
            description: description.into(),
            date: "2024-01-01".into(),
            url: String::new(),
        }
    }

    #[test]
    fn recognises_pages() {
        assert_eq!(Page::from_path("/blog/"), Page::BlogIndex);
        assert_eq!(Page::from_path("/blog"), Page::BlogIndex);
        assert_eq!(Page::from_path("/lab/"), Page::Lab);
        assert_eq!(Page::from_path("/"), Page::Other);
        assert_eq!(Page::from_path("/blog/first-post/"), Page::Other);
        assert_eq!(Page::from_path("/lab/intent/"), Page::Other);
    }

    #[test]
    fn filters_posts_like_the_server() {
        let posts = vec![
            post("Markdown Guide", "syntax"),
            post("First post", "Lorem"),
        ];
        assert_eq!(filter_posts(posts.clone(), "").len(), 2);
        assert_eq!(filter_posts(posts.clone(), "  MARKDOWN ").len(), 1);
        assert_eq!(filter_posts(posts.clone(), "lorem")[0].title, "First post");
        assert!(filter_posts(posts, "nothing").is_empty());
    }

    #[test]
    fn validates_slugs() {
        assert!(is_slug("first-post"));
        assert!(is_slug("truenas_scale"));
        assert!(!is_slug(""));
        assert!(!is_slug("../secret"));
        assert!(!is_slug("a\"] , body [x=\""));
        assert!(!is_slug(&"a".repeat(65)));
    }
}
