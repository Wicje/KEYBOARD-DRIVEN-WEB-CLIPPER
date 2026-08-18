use crate::models::NewClip;
use anyhow::{Context, Result};
use reqwest::blocking::Client;
use scraper::{Html, Selector};
use std::time::Duration;
use url::Url;

pub struct MetadataFetcher {
    client: Client,
}

impl MetadataFetcher {
    pub fn new() -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(12))
            .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36 Clipper/0.1.0")
            .redirect(reqwest::redirect::Policy::limited(10))
            .build()?;
        Ok(Self { client })
    }

    pub fn fetch_and_extract(
        &self,
        url_str: &str,
        user_title: Option<String>,
        user_tags: Vec<String>,
        user_notes: Option<String>,
    ) -> Result<NewClip> {
        let parsed_url = Url::parse(url_str).with_context(|| format!("Invalid URL: '{}'", url_str))?;

        let response_text = match self.client.get(parsed_url.as_str()).send() {
            Ok(resp) if resp.status().is_success() => resp.text().ok(),
            _ => None,
        };

        if let Some(html_content) = response_text {
            let document = Html::parse_document(&html_content);

            // Title extraction
            let extracted_title = Self::extract_title(&document)
                .unwrap_or_else(|| parsed_url.host_str().unwrap_or("Untitled Web Clip").to_string());
            let title = user_title.unwrap_or(extracted_title);

            // Description extraction
            let description = Self::extract_meta(&document, &["description", "og:description", "twitter:description"]);

            // Favicon extraction
            let favicon_url = Self::extract_favicon(&document, &parsed_url);

            // Author extraction
            let author = Self::extract_meta(&document, &["author", "article:author", "twitter:creator"]);

            // Site name extraction
            let site_name = Self::extract_meta(&document, &["og:site_name", "application-name"])
                .or_else(|| parsed_url.host_str().map(|s| s.to_string()));

            // Content text extraction
            let content_text = Self::extract_body_text(&document);
            let reading_time_mins = content_text.as_ref().map(|text| {
                let word_count = text.split_whitespace().count();
                std::cmp::max(1, (word_count / 200) as u32)
            });

            // Combine auto-keywords with user tags
            let mut final_tags = user_tags;
            if let Some(keywords_meta) = Self::extract_meta(&document, &["keywords"]) {
                for kw in keywords_meta.split(',') {
                    let clean = kw.trim().to_lowercase();
                    if !clean.is_empty() && clean.len() <= 20 && !final_tags.contains(&clean) {
                        final_tags.push(clean);
                    }
                }
            }

            Ok(NewClip {
                url: parsed_url.to_string(),
                title,
                description,
                tags: final_tags,
                notes: user_notes,
                content_text,
                screenshot_path: None,
                favicon_url,
                author,
                site_name,
                reading_time_mins,
            })
        } else {
            // Fallback if URL fetch failed or timed out
            let host_title = parsed_url.host_str().unwrap_or("Web Clip").to_string();
            let title = user_title.unwrap_or(host_title);

            Ok(NewClip {
                url: parsed_url.to_string(),
                title,
                description: None,
                tags: user_tags,
                notes: user_notes,
                content_text: None,
                screenshot_path: None,
                favicon_url: Some(format!("{}://{}/favicon.ico", parsed_url.scheme(), parsed_url.host_str().unwrap_or(""))),
                author: None,
                site_name: parsed_url.host_str().map(|s| s.to_string()),
                reading_time_mins: None,
            })
        }
    }

    fn extract_title(document: &Html) -> Option<String> {
        if let Some(title) = Self::extract_meta(document, &["og:title", "twitter:title"]) {
            if !title.trim().is_empty() {
                return Some(title);
            }
        }

        if let Ok(selector) = Selector::parse("title") {
            if let Some(element) = document.select(&selector).next() {
                let text = element.text().collect::<Vec<_>>().join("");
                let clean = text.trim();
                if !clean.is_empty() {
                    return Some(clean.to_string());
                }
            }
        }

        if let Ok(selector) = Selector::parse("h1") {
            if let Some(element) = document.select(&selector).next() {
                let text = element.text().collect::<Vec<_>>().join("");
                let clean = text.trim();
                if !clean.is_empty() {
                    return Some(clean.to_string());
                }
            }
        }

        None
    }

    fn extract_meta(document: &Html, names: &[&str]) -> Option<String> {
        for name in names {
            let selectors = vec![
                format!("meta[name='{}']", name),
                format!("meta[property='{}']", name),
                format!("meta[name='{}' i]", name),
                format!("meta[property='{}' i]", name),
            ];

            for query in selectors {
                let parsed = Selector::parse(&query);
                if let Ok(selector) = parsed {
                    if let Some(element) = document.select(&selector).next() {
                        if let Some(content) = element.value().attr("content") {
                            let clean = content.trim();
                            if !clean.is_empty() {
                                return Some(clean.to_string());
                            }
                        }
                    }
                }
            }
        }
        None
    }

    fn extract_favicon(document: &Html, base_url: &Url) -> Option<String> {
        let rels = ["icon", "shortcut icon", "apple-touch-icon"];
        for rel in rels {
            let query = format!("link[rel*='{}']", rel);
            let parsed = Selector::parse(&query);
            if let Ok(selector) = parsed {
                if let Some(element) = document.select(&selector).next() {
                    if let Some(href) = element.value().attr("href") {
                        if let Ok(resolved) = base_url.join(href) {
                            return Some(resolved.to_string());
                        }
                    }
                }
            }
        }
        Some(format!("{}://{}/favicon.ico", base_url.scheme(), base_url.host_str().unwrap_or("")))
    }

    fn extract_body_text(document: &Html) -> Option<String> {
        let mut paragraphs = Vec::new();
        let target_selectors = ["article", "main", "p", "h1", "h2", "h3", "li"];

        for tag in target_selectors {
            let parsed = Selector::parse(tag);
            if let Ok(selector) = parsed {
                for element in document.select(&selector) {
                    let text = element.text().collect::<Vec<_>>().join(" ");
                    let clean = text.trim();
                    if clean.len() > 30 && !paragraphs.contains(&clean.to_string()) {
                        paragraphs.push(clean.to_string());
                    }
                }
            }
        }

        if paragraphs.is_empty() {
            None
        } else {
            let combined = paragraphs.join("\n\n");
            if combined.len() > 10000 {
                Some(format!("{}...", &combined[..10000]))
            } else {
                Some(combined)
            }
        }
    }
}
