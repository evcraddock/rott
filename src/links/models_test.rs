#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_link_new() {
        let title = "Test Title".to_string();
        let source = Some("https://example.com".to_string());
        let author = vec!["Test Author".to_string()];
        let published = Some(NaiveDate::from_ymd_opt(2025, 1, 1).unwrap());
        let created = NaiveDate::from_ymd_opt(2025, 1, 2).unwrap();
        let description = Some("Test Description".to_string());
        let tags = vec!["test".to_string(), "example".to_string()];

        let link = Link::new(
            title.clone(),
            source.clone(),
            author.clone(),
            published,
            created,
            description.clone(),
            tags.clone(),
        );

        assert_eq!(link.title, title);
        assert_eq!(link.source, source);
        assert_eq!(link.author, author);
        assert_eq!(link.published, published);
        assert_eq!(link.created, created);
        assert_eq!(link.description, description);
        assert_eq!(link.tags, tags);
        assert_eq!(link.file_path, None);
        assert_eq!(link.content, None);
    }

    #[test]
    fn test_link_default() {
        let link = Link::default();

        assert_eq!(link.title, "");
        assert_eq!(link.source, None);
        assert!(link.author.is_empty());
        assert_eq!(link.published, None);
        // created should be today's date
        assert_eq!(link.description, None);
        assert!(link.tags.is_empty());
        assert_eq!(link.file_path, None);
        assert_eq!(link.content, None);
    }
}
