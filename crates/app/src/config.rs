//! Typed view of `content/config.json`, the file `scripts/sync.ts` generates
//! from `content/data.yml`, `content/systems.yml` and GitHub.

use serde::Deserialize;
use std::sync::LazyLock;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub site: Site,
    pub current: Current,
    pub systems: Vec<System>,
    pub sections: Sections,
}

#[derive(Debug, Deserialize)]
pub struct Site {
    pub title: String,
    pub description: String,
    pub url: String,
    pub author: String,
    #[serde(default)]
    pub handle_suffix: String,
    pub image: String,
    pub image_width: u32,
    pub image_height: u32,
    pub theme_color: String,
    pub job_title: String,
    #[serde(default)]
    pub social: Social,
    #[serde(default)]
    pub twitter: String,
    pub locale: String,
    pub default_theme: String,
    #[serde(default)]
    pub footer: Footer,
}

#[derive(Debug, Default, Deserialize)]
pub struct Social {
    #[serde(default)]
    pub github: String,
}

#[derive(Debug, Deserialize)]
pub struct Footer {
    pub status: String,
    pub errors: u32,
    pub warnings: u32,
    pub uptime: String,
}

impl Default for Footer {
    fn default() -> Self {
        Self {
            status: "session closed".into(),
            errors: 0,
            warnings: 0,
            uptime: "0y".into(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Current {
    pub booted_at: String,
    pub host: String,
    pub cpu: String,
    pub gpu: String,
    pub memory: String,
    pub resolution: String,
}

#[derive(Debug, Deserialize)]
pub struct System {
    pub slug: String,
    pub os: String,
    pub pkg: String,
    pub kernel: String,
    pub shell: String,
    pub de: String,
    pub wm: String,
    pub theme: String,
    pub terminal: String,
    pub pkgs: String,
    pub font: String,
    pub colors: Colors,
}

#[derive(Debug, Deserialize)]
pub struct Colors {
    pub bg: String,
    pub panel: String,
    pub border: String,
    pub fg: String,
    pub dim: String,
    pub accent: String,
    pub c1: String,
    pub c2: String,
    pub c3: String,
}

#[derive(Debug, Deserialize)]
pub struct Sections {
    pub about: About,
    pub projects: Projects,
    pub uses: Uses,
    pub contact: Contact,
}

#[derive(Debug, Deserialize)]
pub struct About {
    pub command: String,
    pub status: GitStatus,
    pub bio: Vec<BioItem>,
    pub paths: Vec<PathEntry>,
}

#[derive(Debug, Deserialize)]
pub struct GitStatus {
    pub branch: String,
    pub since: u32,
    #[serde(default)]
    pub ahead: u32,
    #[serde(default)]
    pub behind: u32,
    pub modified: u32,
    pub untracked: u32,
}

#[derive(Debug, Deserialize)]
pub struct BioItem {
    pub icon: String,
    pub text: String,
}

#[derive(Debug, Deserialize)]
pub struct PathEntry {
    pub name: String,
    #[serde(default)]
    pub current: bool,
}

#[derive(Debug, Deserialize)]
pub struct Projects {
    pub command: String,
    pub root: String,
    pub projects: Vec<Project>,
}

#[derive(Debug, Deserialize)]
pub struct Project {
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub repo: String,
    pub stars: String,
    pub updated: String,
    pub size: String,
    pub date: String,
    pub url: String,
    #[serde(default)]
    pub homepage: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Uses {
    pub command: String,
    pub cpu: Cpu,
    pub memory: Memory,
    pub hardware: Hardware,
    pub peripherals: Peripherals,
}

#[derive(Debug, Deserialize)]
pub struct Cpu {
    pub usage: Vec<Usage>,
}

#[derive(Debug, Deserialize)]
pub struct Usage {
    pub label: String,
    pub percent: u32,
    pub detail: String,
    #[serde(default)]
    pub note: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Memory {
    pub used: f64,
    pub total: f64,
    pub theme: String,
    #[serde(rename = "themePercent")]
    pub theme_percent: u32,
}

#[derive(Debug, Deserialize)]
pub struct Hardware {
    pub device: String,
    pub ram: String,
    pub os: String,
}

#[derive(Debug, Deserialize)]
pub struct Peripherals {
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub keyboard: Option<String>,
    #[serde(default)]
    pub mouse: Option<String>,
    #[serde(default)]
    pub headphones: Option<String>,
    #[serde(default)]
    pub monitor: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Contact {
    pub command: String,
    pub email: String,
    pub inbox: Vec<Mail>,
    pub hints: Vec<Hint>,
}

#[derive(Debug, Deserialize)]
pub struct Mail {
    pub from: String,
    pub date: String,
    pub href: String,
}

#[derive(Debug, Deserialize)]
pub struct Hint {
    pub key: String,
    pub action: String,
}

static CONFIG: LazyLock<Config> = LazyLock::new(|| {
    serde_json::from_str(include_str!("../../../content/config.json"))
        .expect("content/config.json does not match the Config schema")
});

/// The site configuration, parsed once.
pub fn config() -> &'static Config {
    &CONFIG
}

/// Theme slugs in the order the theme menu lists them.
pub fn themes() -> Vec<&'static str> {
    let mut slugs: Vec<&str> = config().systems.iter().map(|s| s.slug.as_str()).collect();
    slugs.sort_unstable();
    slugs
}

/// Web font that stands in for each distro's terminal font.
pub fn font_family(font: &str) -> &'static str {
    match font {
        "Ubuntu Mono" => "Ubuntu Mono",
        "Hack" => "Fira Code",
        "Inter" | "Cantarell" => "Inter",
        "Noto Sans" => "Noto Sans",
        "Noto Sans Mono" => "Noto Sans Mono",
        "Roboto Mono" => "Roboto Mono",
        _ => "JetBrains Mono",
    }
}

/// Absolute URL for a site-relative path.
pub fn abs(path: &str) -> String {
    format!("{}{}", config().site.url.trim_end_matches('/'), path)
}
