use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Clip {
    pub id: i64,
    pub url: String,
    pub title: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub notes: Option<String>,
    pub content_text: Option<String>,
    pub screenshot_path: Option<String>,
    pub favicon_url: Option<String>,
    pub author: Option<String>,
    pub site_name: Option<String>,
    pub reading_time_mins: Option<u32>,
    pub date_saved: DateTime<Utc>,
    pub date_updated: DateTime<Utc>,
}

impl Clip {
    pub fn tags_string(&self) -> String {
        self.tags.join(", ")
    }

    pub fn display_title(&self) -> &str {
        if self.title.trim().is_empty() {
            &self.url
        } else {
            &self.title
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewClip {
    pub url: String,
    pub title: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub notes: Option<String>,
    pub content_text: Option<String>,
    pub screenshot_path: Option<String>,
    pub favicon_url: Option<String>,
    pub author: Option<String>,
    pub site_name: Option<String>,
    pub reading_time_mins: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateClip {
    pub id: i64,
    pub title: Option<String>,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct ClipFilter {
    pub query: Option<String>,
    pub tag: Option<String>,
    pub from_date: Option<DateTime<Utc>>,
    pub to_date: Option<DateTime<Utc>>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagCount {
    pub tag: String,
    pub count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum ExportFormat {
    Json,
    Csv,
    Html,
}

impl std::fmt::Display for ExportFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExportFormat::Json => write!(f, "json"),
            ExportFormat::Csv => write!(f, "csv"),
            ExportFormat::Html => write!(f, "html"),
        }
    }
}
