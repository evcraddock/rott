//! URL metadata fetching
//!
//! Fetches title, description, and author from URLs when creating links.

use crate::app::UrlMetadata;
use scraper::{Html, Selector};
use std::time::Duration;

/// Fetch timeout in seconds
const FETCH_TIMEOUT: u64 = 10;

/// Fetch metadata from a URL (async version)
///
/// Returns None on failure (graceful degradation).
pub async fn fetch_metadata(url: &str) -> Option<UrlMetadata> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(FETCH_TIMEOUT))
        .user_agent("Mozilla/5.0 (compatible; ROTT/1.0)")
        .build()
        .ok()?;

    let response = client.get(url).send().await.ok()?;

    if !response.status().is_success() {
        return None;
    }

    let html = response.text().await.ok()?;
    Some(parse_metadata(&html))
}

/// Parse metadata from HTML content
fn parse_metadata(html: &str) -> UrlMetadata {
    let document = Html::parse_document(html);

    let title = extract_title(&document);
    let description = extract_description(&document);
    let author = extract_author(&document);

    UrlMetadata {
        title,
        description,
        author,
    }
}

/// Extract title from HTML
fn extract_title(document: &Html) -> Option<String> {
    // Try og:title first
    if let Some(og_title) = extract_meta_content(document, "og:title") {
        return Some(og_title);
    }

    // Try twitter:title
    if let Some(twitter_title) = extract_meta_content(document, "twitter:title") {
        return Some(twitter_title);
    }

    // Fall back to <title> tag
    let selector = Selector::parse("title").ok()?;
    document
        .select(&selector)
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .filter(|s| !s.is_empty())
}

/// Extract description from HTML
fn extract_description(document: &Html) -> Option<String> {
    // Try og:description first
    if let Some(og_desc) = extract_meta_content(document, "og:description") {
        return Some(og_desc);
    }

    // Try twitter:description
    if let Some(twitter_desc) = extract_meta_content(document, "twitter:description") {
        return Some(twitter_desc);
    }

    // Fall back to meta description
    let selector = Selector::parse(r#"meta[name="description"]"#).ok()?;
    document
        .select(&selector)
        .next()
        .and_then(|el| el.value().attr("content"))
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

/// Extract author from HTML
fn extract_author(document: &Html) -> Vec<String> {
    let mut authors = Vec::new();

    // Try article:author
    if let Some(author) = extract_meta_content(document, "article:author") {
        authors.push(author);
    }

    // Try author meta tag
    if let Ok(selector) = Selector::parse(r#"meta[name="author"]"#) {
        for el in document.select(&selector) {
            if let Some(content) = el.value().attr("content") {
                let content = content.trim();
                if !content.is_empty() && !authors.contains(&content.to_string()) {
                    authors.push(content.to_string());
                }
            }
        }
    }

    // Try dc.creator
    if let Ok(selector) = Selector::parse(r#"meta[name="dc.creator"]"#) {
        for el in document.select(&selector) {
            if let Some(content) = el.value().attr("content") {
                let content = content.trim();
                if !content.is_empty() && !authors.contains(&content.to_string()) {
                    authors.push(content.to_string());
                }
            }
        }
    }

    authors
}

/// Extract content from a meta tag by property or name
fn extract_meta_content(document: &Html, property: &str) -> Option<String> {
    // Try property attribute (for Open Graph)
    let property_selector = format!(r#"meta[property="{}"]"#, property);
    if let Ok(selector) = Selector::parse(&property_selector) {
        if let Some(el) = document.select(&selector).next() {
            if let Some(content) = el.value().attr("content") {
                let content = content.trim();
                if !content.is_empty() {
                    return Some(content.to_string());
                }
            }
        }
    }

    // Try name attribute
    let name_selector = format!(r#"meta[name="{}"]"#, property);
    if let Ok(selector) = Selector::parse(&name_selector) {
        if let Some(el) = document.select(&selector).next() {
            if let Some(content) = el.value().attr("content") {
                let content = content.trim();
                if !content.is_empty() {
                    return Some(content.to_string());
                }
            }
        }
    }

    None
}
