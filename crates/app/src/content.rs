//! Blog posts and distro logos, embedded at compile time from `content/`.

use include_dir::{include_dir, Dir};
use pulldown_cmark::{html, Options, Parser};
use std::sync::LazyLock;

static BLOG: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../../content/blog");
static LOGOS: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../../content/logos");

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Date {
    pub year: i32,
    pub month: u32,
    pub day: u32,
}

const MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];
const WEEKDAYS: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];

impl Date {
    /// Parses `Jul 08 2022` (the front matter format) or `2022-07-08`.
    pub fn parse(text: &str) -> Option<Self> {
        let text = text.trim();
        let iso: Vec<&str> = text.split('-').collect();
        if iso.len() == 3 {
            return Some(Self {
                year: iso[0].parse().ok()?,
                month: iso[1].parse().ok()?,
                day: iso[2].get(..2)?.parse().ok()?,
            });
        }
        let mut parts = text.split_whitespace();
        let month = parts.next()?;
        let month = MONTHS.iter().position(|m| month.starts_with(m))? as u32 + 1;
        let day = parts.next()?.trim_end_matches(',').parse().ok()?;
        let year = parts.next()?.parse().ok()?;
        Some(Self { year, month, day })
    }

    /// `2022-07-08`
    pub fn iso(&self) -> String {
        format!("{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }

    /// `Jul 8, 2022`
    pub fn human(&self) -> String {
        format!(
            "{} {}, {}",
            MONTHS[(self.month - 1) as usize],
            self.day,
            self.year
        )
    }

    /// `Fri, 08 Jul 2022 00:00:00 GMT`, as RSS wants it.
    pub fn rfc822(&self) -> String {
        // Sakamoto's day-of-week algorithm.
        const T: [i32; 12] = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
        let y = if self.month < 3 {
            self.year - 1
        } else {
            self.year
        };
        let weekday =
            (y + y / 4 - y / 100 + y / 400 + T[(self.month - 1) as usize] + self.day as i32)
                .rem_euclid(7);
        format!(
            "{}, {:02} {} {} 00:00:00 GMT",
            WEEKDAYS[weekday as usize],
            self.day,
            MONTHS[(self.month - 1) as usize],
            self.year
        )
    }
}

#[derive(Debug)]
pub struct Post {
    pub slug: String,
    pub title: String,
    pub description: String,
    pub date: Date,
    pub hero_image: Option<String>,
    /// Markdown source without the front matter.
    pub body: String,
    /// Rendered HTML.
    pub html: String,
}

fn unquote(value: &str) -> String {
    let value = value.trim();
    let quoted = value.len() >= 2
        && ((value.starts_with('"') && value.ends_with('"'))
            || (value.starts_with('\'') && value.ends_with('\'')));
    if quoted {
        value[1..value.len() - 1].to_string()
    } else {
        value.to_string()
    }
}

fn render_markdown(source: &str) -> String {
    let options = Options::ENABLE_TABLES
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS;
    let mut out = String::with_capacity(source.len() * 3 / 2);
    html::push_html(&mut out, Parser::new_ext(source, options));
    out
}

fn parse_post(slug: &str, source: &str) -> Option<Post> {
    let source = source.replace("\r\n", "\n");
    let rest = source.strip_prefix("---\n")?;
    let (front, body) = rest.split_once("\n---")?;
    let body = body.trim_start_matches('-').trim().to_string();

    let (mut title, mut description, mut date, mut hero_image) = (None, None, None, None);
    for line in front.lines() {
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        let value = unquote(value);
        match key.trim() {
            "title" => title = Some(value),
            "description" => description = Some(value),
            "pubDate" => date = Date::parse(&value),
            "heroImage" => hero_image = Some(value),
            _ => {}
        }
    }

    Some(Post {
        slug: slug.to_string(),
        title: title?,
        description: description.unwrap_or_default(),
        date: date?,
        hero_image,
        html: render_markdown(&body),
        body,
    })
}

static POSTS: LazyLock<Vec<Post>> = LazyLock::new(|| {
    let mut posts: Vec<Post> = BLOG
        .files()
        .filter_map(|file| {
            let path = file.path();
            if path.extension()?.to_str()? != "md" {
                return None;
            }
            let slug = path.file_stem()?.to_str()?;
            parse_post(slug, file.contents_utf8()?)
        })
        .collect();
    posts.sort_by(|a, b| b.date.cmp(&a.date).then_with(|| a.slug.cmp(&b.slug)));
    posts
});

/// Every blog post, newest first.
pub fn posts() -> &'static [Post] {
    &POSTS
}

pub fn post(slug: &str) -> Option<&'static Post> {
    posts().iter().find(|post| post.slug == slug)
}

/// Posts whose title or description contains `query` (case-insensitive).
pub fn search_posts(query: &str) -> Vec<&'static Post> {
    let query = query.trim().to_lowercase();
    posts()
        .iter()
        .filter(|post| {
            query.is_empty()
                || post.title.to_lowercase().contains(&query)
                || post.description.to_lowercase().contains(&query)
        })
        .collect()
}

/// The fastfetch ASCII logo for a distro slug, if it is bundled.
pub fn logo(slug: &str) -> Option<&'static str> {
    LOGOS.get_file(format!("{slug}.txt"))?.contents_utf8()
}

pub fn escape_html(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(ch),
        }
    }
    out
}

/// Turns a fastfetch logo (`$1`..`$9` switch colour, `$$` is a literal `$`)
/// into spans using the three theme logo colours.
pub fn logo_html(source: &str) -> String {
    const CLASSES: [&str; 3] = ["text-c1", "text-c2", "text-c3"];
    let mut color = 1usize;
    let normalized = source.replace("\r\n", "\n");

    let lines: Vec<String> = normalized
        .trim_matches('\n')
        .split('\n')
        .map(|line| {
            let mut html = String::new();
            let mut buffer = String::new();
            let flush = |buffer: &mut String, html: &mut String, color: usize| {
                if buffer.is_empty() {
                    return;
                }
                let class = CLASSES[color.clamp(1, CLASSES.len()) - 1];
                html.push_str(&format!(
                    "<span class=\"{class}\">{}</span>",
                    escape_html(buffer)
                ));
                buffer.clear();
            };

            let chars: Vec<char> = line.chars().collect();
            let mut i = 0;
            while i < chars.len() {
                if chars[i] != '$' || i + 1 >= chars.len() {
                    buffer.push(chars[i]);
                    i += 1;
                    continue;
                }
                let marker = chars[i + 1];
                i += 2;
                match marker {
                    '$' => buffer.push('$'),
                    '1'..='9' => {
                        flush(&mut buffer, &mut html, color);
                        color = marker.to_digit(10).unwrap_or(1) as usize;
                    }
                    other => {
                        buffer.push('$');
                        buffer.push(other);
                    }
                }
            }
            flush(&mut buffer, &mut html, color);
            html
        })
        .collect();

    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_both_date_formats() {
        let expected = Date {
            year: 2022,
            month: 7,
            day: 8,
        };
        assert_eq!(Date::parse("Jul 08 2022"), Some(expected));
        assert_eq!(Date::parse("2022-07-08"), Some(expected));
        assert_eq!(Date::parse("nonsense"), None);
    }

    #[test]
    fn formats_dates() {
        let date = Date {
            year: 2022,
            month: 7,
            day: 8,
        };
        assert_eq!(date.iso(), "2022-07-08");
        assert_eq!(date.human(), "Jul 8, 2022");
        assert_eq!(date.rfc822(), "Fri, 08 Jul 2022 00:00:00 GMT");
        let leap = Date {
            year: 2024,
            month: 2,
            day: 29,
        };
        assert_eq!(leap.rfc822(), "Thu, 29 Feb 2024 00:00:00 GMT");
    }

    #[test]
    fn loads_posts_newest_first() {
        let posts = posts();
        assert!(!posts.is_empty());
        assert!(posts.windows(2).all(|pair| pair[0].date >= pair[1].date));
        assert!(posts
            .iter()
            .all(|p| !p.title.is_empty() && !p.html.is_empty()));
    }

    #[test]
    fn search_is_case_insensitive() {
        assert_eq!(search_posts("").len(), posts().len());
        assert!(search_posts("MARKDOWN")
            .iter()
            .any(|p| p.slug == "markdown-style-guide"));
        assert!(search_posts("no such post anywhere").is_empty());
    }

    #[test]
    fn every_system_has_a_logo() {
        for system in &crate::config::config().systems {
            assert!(
                logo(&system.slug).is_some(),
                "missing logo for {}",
                system.slug
            );
        }
    }

    #[test]
    fn colours_logo_segments() {
        assert_eq!(
            logo_html("$1ab$2c<\n$$5"),
            "<span class=\"text-c1\">ab</span><span class=\"text-c2\">c&lt;</span>\n<span class=\"text-c2\">$5</span>"
        );
    }
}
