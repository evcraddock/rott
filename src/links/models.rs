use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Link {
    pub title: String,
    pub source: Option<String>,
    #[serde(default = "Vec::new")]
    pub author: Vec<String>,
    pub published: Option<NaiveDate>,
    #[serde(default = "default_created")]
    pub created: NaiveDate,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub content: Option<String>,
    pub file_path: Option<String>,
}

fn default_title() -> String {
    "No Title".to_string()
}

fn default_created() -> NaiveDate {
    chrono::Local::now().date_naive()
}

impl Link {
    pub fn new(
        title: String,
        source: Option<String>,
        author: Vec<String>,
        published: Option<NaiveDate>,
        created: NaiveDate,
        description: Option<String>,
        tags: Vec<String>,
    ) -> Self {
        Self {
            title,
            source,
            author,
            published,
            created,
            description,
            tags,
            content: None,
            file_path: None,
        }
    }

    pub fn default() -> Self {
        Self {
            title: String::new(),
            source: None,
            author: Vec::new(),
            published: None,
            created: NaiveDate::from_ymd_opt(1970, 1, 1).unwrap(),
            description: None,
            tags: Vec::new(),
            content: None,
            file_path: None,
        }
    }
}

#[derive(Debug)]
pub struct LinkError {
    pub message: String,
}

impl std::error::Error for LinkError {}
impl std::fmt::Display for LinkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl From<std::io::Error> for LinkError {
    fn from(error: std::io::Error) -> Self {
        LinkError {
            message: error.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_link_creation() {
        let link = Link::new(
            "Test Title".to_string(),
            Some("https://example.com".to_string()),
            vec!["Test Author".to_string()],
            None,
            NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            Some("Test Description".to_string()),
            vec!["test".to_string()],
        );

        assert_eq!(link.title, "Test Title");
        assert_eq!(link.source, Some("https://example.com".to_string()));
        assert_eq!(link.author, vec!["Test Author".to_string()]);
        assert_eq!(link.created, NaiveDate::from_ymd_opt(2025, 1, 1).unwrap());
    }
}
