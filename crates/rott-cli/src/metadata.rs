//! URL metadata fetching
//!
//! Fetches title, description, and author from URLs when creating links.

use anyhow::Result;
use scraper::{Html, Selector};
use std::time::Duration;

/// Metadata extracted from a URL
#[derive(Debug, Clone, Default)]
pub struct UrlMetadata {
    pub title: Option<String>,
    pub description: Option<String>,
    pub author: Vec<String>,
}

/// Fetch timeout in seconds
const FETCH_TIMEOUT: u64 = 10;

/// Fetch metadata from a URL (async)
///
/// Returns empty metadata on failure (graceful degradation).
pub async fn fetch_metadata(url: &str) -> UrlMetadata {
    fetch_metadata_inner(url).await.unwrap_or_default()
}

/// Inner fetch function that can fail
async fn fetch_metadata_inner(url: &str) -> Result<UrlMetadata> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(FETCH_TIMEOUT))
        .user_agent("Mozilla/5.0 (compatible; ROTT/1.0)")
        .build()?;

    let response = client.get(url).send().await?;

    if !response.status().is_success() {
        return Ok(UrlMetadata::default());
    }

    let html = response.text().await?;
    Ok(parse_metadata(&html))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_metadata_basic() {
        let html = r#"
            <!DOCTYPE html>
            <html>
            <head>
                <title>Test Page</title>
                <meta name="description" content="A test description">
                <meta name="author" content="Test Author">
            </head>
            <body></body>
            </html>
        "#;

        let metadata = parse_metadata(html);
        assert_eq!(metadata.title, Some("Test Page".to_string()));
        assert_eq!(metadata.description, Some("A test description".to_string()));
        assert_eq!(metadata.author, vec!["Test Author".to_string()]);
    }

    #[test]
    fn test_parse_metadata_opengraph() {
        let html = r#"
            <!DOCTYPE html>
            <html>
            <head>
                <title>Fallback Title</title>
                <meta property="og:title" content="OG Title">
                <meta property="og:description" content="OG Description">
            </head>
            <body></body>
            </html>
        "#;

        let metadata = parse_metadata(html);
        // OG takes precedence
        assert_eq!(metadata.title, Some("OG Title".to_string()));
        assert_eq!(metadata.description, Some("OG Description".to_string()));
    }

    #[test]
    fn test_parse_metadata_empty() {
        let html = "<html><head></head><body></body></html>";
        let metadata = parse_metadata(html);
        assert!(metadata.title.is_none());
        assert!(metadata.description.is_none());
        assert!(metadata.author.is_empty());
    }

    #[test]
    fn test_parse_metadata_multiple_authors() {
        let html = r#"
            <!DOCTYPE html>
            <html>
            <head>
                <meta name="author" content="Author One">
                <meta name="dc.creator" content="Author Two">
            </head>
            <body></body>
            </html>
        "#;

        let metadata = parse_metadata(html);
        assert_eq!(metadata.author.len(), 2);
        assert!(metadata.author.contains(&"Author One".to_string()));
        assert!(metadata.author.contains(&"Author Two".to_string()));
    }
}
