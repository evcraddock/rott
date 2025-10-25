use reqwest::blocking::Client;
use scraper::{Html, Selector};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct PageMetadata {
    pub title: Option<String>,
    pub description: Option<String>,
    pub author: Vec<String>,
}

pub fn fetch_url_metadata(url: &str) -> Result<PageMetadata, Box<dyn std::error::Error>> {
    // Create HTTP client with timeout
    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .user_agent("Mozilla/5.0 (compatible; rott/0.1.0)")
        .build()?;

    // Fetch the page
    let response = client.get(url).send()?;
    let html_content = response.text()?;

    // Parse HTML
    let document = Html::parse_document(&html_content);

    // Extract metadata
    let title = extract_title(&document, url);
    let description = extract_description(&document);
    let author = extract_author(&document);

    Ok(PageMetadata {
        title,
        description,
        author,
    })
}

fn extract_title(document: &Html, fallback_url: &str) -> Option<String> {
    // Try Open Graph title first
    if let Some(title) = extract_meta_content(document, "meta[property='og:title']") {
        return Some(title);
    }

    // Try Twitter title
    if let Some(title) = extract_meta_content(document, "meta[name='twitter:title']") {
        return Some(title);
    }

    // Try standard title tag
    if let Ok(selector) = Selector::parse("title") {
        if let Some(element) = document.select(&selector).next() {
            let title = element.text().collect::<String>().trim().to_string();
            if !title.is_empty() {
                return Some(title);
            }
        }
    }

    // Fallback to URL
    Some(fallback_url.to_string())
}

fn extract_description(document: &Html) -> Option<String> {
    // Try Open Graph description
    if let Some(desc) = extract_meta_content(document, "meta[property='og:description']") {
        return Some(desc);
    }

    // Try Twitter description
    if let Some(desc) = extract_meta_content(document, "meta[name='twitter:description']") {
        return Some(desc);
    }

    // Try standard description
    if let Some(desc) = extract_meta_content(document, "meta[name='description']") {
        return Some(desc);
    }

    None
}

fn extract_author(document: &Html) -> Vec<String> {
    let mut authors = Vec::new();

    // Try standard author meta tag
    if let Some(author) = extract_meta_content(document, "meta[name='author']") {
        authors.push(author);
        return authors;
    }

    // Try Open Graph article:author
    if let Some(author) = extract_meta_content(document, "meta[property='article:author']") {
        authors.push(author);
        return authors;
    }

    // Try Twitter creator
    if let Some(author) = extract_meta_content(document, "meta[name='twitter:creator']") {
        // Twitter handles start with @, remove it
        let clean_author = author.trim_start_matches('@').to_string();
        authors.push(clean_author);
        return authors;
    }

    // Try rel="author" link
    if let Ok(selector) = Selector::parse("a[rel='author']") {
        for element in document.select(&selector) {
            if let Some(author_name) = element.text().next() {
                let trimmed = author_name.trim().to_string();
                if !trimmed.is_empty() {
                    authors.push(trimmed);
                }
            }
        }
        if !authors.is_empty() {
            return authors;
        }
    }

    // Try JSON-LD structured data
    if let Ok(selector) = Selector::parse("script[type='application/ld+json']") {
        for element in document.select(&selector) {
            if let Some(json_text) = element.text().next() {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(json_text) {
                    // Check for author in Article
                    if let Some(author_data) = json.get("author") {
                        if let Some(name) = author_data.get("name") {
                            if let Some(name_str) = name.as_str() {
                                authors.push(name_str.to_string());
                                return authors;
                            }
                        }
                    }
                }
            }
        }
    }

    authors
}

fn extract_meta_content(document: &Html, selector_str: &str) -> Option<String> {
    if let Ok(selector) = Selector::parse(selector_str) {
        if let Some(element) = document.select(&selector).next() {
            if let Some(content) = element.value().attr("content") {
                let trimmed = content.trim().to_string();
                if !trimmed.is_empty() {
                    return Some(trimmed);
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
    fn test_extract_title_from_title_tag() {
        let html = r#"
            <!DOCTYPE html>
            <html>
            <head>
                <title>Test Page Title</title>
            </head>
            </html>
        "#;
        let document = Html::parse_document(html);
        let title = extract_title(&document, "https://example.com");
        assert_eq!(title, Some("Test Page Title".to_string()));
    }

    #[test]
    fn test_extract_title_from_og_title() {
        let html = r#"
            <!DOCTYPE html>
            <html>
            <head>
                <meta property="og:title" content="Open Graph Title">
                <title>Regular Title</title>
            </head>
            </html>
        "#;
        let document = Html::parse_document(html);
        let title = extract_title(&document, "https://example.com");
        assert_eq!(title, Some("Open Graph Title".to_string()));
    }

    #[test]
    fn test_extract_description_from_meta() {
        let html = r#"
            <!DOCTYPE html>
            <html>
            <head>
                <meta name="description" content="This is a test description">
            </head>
            </html>
        "#;
        let document = Html::parse_document(html);
        let description = extract_description(&document);
        assert_eq!(description, Some("This is a test description".to_string()));
    }

    #[test]
    fn test_extract_description_prefers_og() {
        let html = r#"
            <!DOCTYPE html>
            <html>
            <head>
                <meta property="og:description" content="OG Description">
                <meta name="description" content="Regular Description">
            </head>
            </html>
        "#;
        let document = Html::parse_document(html);
        let description = extract_description(&document);
        assert_eq!(description, Some("OG Description".to_string()));
    }

    #[test]
    fn test_extract_author_from_meta() {
        let html = r#"
            <!DOCTYPE html>
            <html>
            <head>
                <meta name="author" content="John Doe">
            </head>
            </html>
        "#;
        let document = Html::parse_document(html);
        let authors = extract_author(&document);
        assert_eq!(authors, vec!["John Doe".to_string()]);
    }

    #[test]
    fn test_extract_author_from_twitter() {
        let html = r#"
            <!DOCTYPE html>
            <html>
            <head>
                <meta name="twitter:creator" content="@johndoe">
            </head>
            </html>
        "#;
        let document = Html::parse_document(html);
        let authors = extract_author(&document);
        assert_eq!(authors, vec!["johndoe".to_string()]);
    }

    #[test]
    fn test_extract_author_from_article_author() {
        let html = r#"
            <!DOCTYPE html>
            <html>
            <head>
                <meta property="article:author" content="Jane Smith">
            </head>
            </html>
        "#;
        let document = Html::parse_document(html);
        let authors = extract_author(&document);
        assert_eq!(authors, vec!["Jane Smith".to_string()]);
    }

    #[test]
    fn test_no_author_returns_empty() {
        let html = r#"
            <!DOCTYPE html>
            <html>
            <head>
                <title>No Author Page</title>
            </head>
            </html>
        "#;
        let document = Html::parse_document(html);
        let authors = extract_author(&document);
        assert!(authors.is_empty());
    }

    #[test]
    fn test_title_fallback_to_url() {
        let html = r#"
            <!DOCTYPE html>
            <html>
            <head>
            </head>
            </html>
        "#;
        let document = Html::parse_document(html);
        let title = extract_title(&document, "https://example.com/page");
        assert_eq!(title, Some("https://example.com/page".to_string()));
    }
}
