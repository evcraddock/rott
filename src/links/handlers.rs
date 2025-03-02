use std::error::Error;

use super::Link;

pub struct LinkHandler {
    links: Vec<Link>,
    #[test]
    fn test_update_link_not_found() {
        let mut handler = LinkHandler::new();
        
        // Create a test link
        let link = Link::new(
            "Original Title".to_string(),
            Some("https://example.com".to_string()),
            vec!["Author".to_string()],
            None,
            NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            Some("Description".to_string()),
            vec!["tag".to_string()],
        );
        
        handler.add_link(link);
        
        // Create an updated link with a different title
        let updated_link = Link::new(
            "Updated Title".to_string(),
            Some("https://updated.com".to_string()),
            vec!["Updated Author".to_string()],
            None,
            NaiveDate::from_ymd_opt(2025, 1, 2).unwrap(),
            Some("Updated Description".to_string()),
            vec!["updated".to_string()],
        );
        
        // Try to update a link that doesn't exist
        let result = handler.update_link("Nonexistent Title", updated_link);
        
        // Verify the operation failed
        assert!(result.is_err());
        
        // Verify the original link is still there
        assert_eq!(handler.get_links().len(), 1);
        let original = handler.get_link_by_title("Original Title").unwrap();
        assert_eq!(original.description, Some("Description".to_string()));
    }

    #[test]
    fn test_delete_link_not_found() {
        let mut handler = LinkHandler::new();
        
        // Create a test link
        let link = Link::new(
            "Test Title".to_string(),
            Some("https://example.com".to_string()),
            vec!["Author".to_string()],
            None,
            NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            Some("Description".to_string()),
            vec!["tag".to_string()],
        );
        
        handler.add_link(link);
        
        // Try to delete a link that doesn't exist
        let result = handler.delete_link("Nonexistent Title");
        
        // Verify the operation failed
        assert!(result.is_err());
        
        // Verify the original link is still there
        assert_eq!(handler.get_links().len(), 1);
    }

    #[test]
    fn test_empty_handler() {
        let handler = LinkHandler::new();
        
        // Verify operations on an empty handler
        assert!(handler.get_links().is_empty());
        assert!(handler.get_link_by_title("Any Title").is_none());
        assert!(handler.get_links_by_tag("Any Tag").is_empty());
        assert!(handler.get_links_by_author("Any Author").is_empty());
    }

    #[test]
    fn test_update_link_success() {
        let mut handler = LinkHandler::new();
        
        // Create a test link
        let link = Link::new(
            "Test Title".to_string(),
            Some("https://example.com".to_string()),
            vec!["Author".to_string()],
            None,
            NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            Some("Description".to_string()),
            vec!["tag".to_string()],
        );
        
        handler.add_link(link);
        
        // Create an updated link
        let updated_link = Link::new(
            "Test Title".to_string(), // Same title
            Some("https://updated.com".to_string()),
            vec!["Updated Author".to_string()],
            None,
            NaiveDate::from_ymd_opt(2025, 1, 2).unwrap(),
            Some("Updated Description".to_string()),
            vec!["updated".to_string()],
        );
        
        // Update the link
        let result = handler.update_link("Test Title", updated_link);
        
        // Verify the operation succeeded
        assert!(result.is_ok());
        
        // Verify the link was updated
        let updated = handler.get_link_by_title("Test Title").unwrap();
        assert_eq!(updated.description, Some("Updated Description".to_string()));
        assert_eq!(updated.source, Some("https://updated.com".to_string()));
    }
}

impl LinkHandler {
    pub fn new() -> Self {
        Self { links: Vec::new() }
    }

    pub fn add_link(&mut self, link: Link) {
        self.links.push(link);
    }

    pub fn get_links(&self) -> &Vec<Link> {
        &self.links
    }

    pub fn get_link_by_title(&self, title: &str) -> Option<&Link> {
        self.links.iter().find(|link| link.title == title)
    }

    pub fn get_links_by_tag(&self, tag: &str) -> Vec<&Link> {
        self.links
            .iter()
            .filter(|link| link.tags.contains(&tag.to_string()))
            .collect()
    }

    pub fn get_links_by_author(&self, author: &str) -> Vec<&Link> {
        self.links
            .iter()
            .filter(|link| link.author.contains(&author.to_string()))
            .collect()
    }

    pub fn update_link(&mut self, title: &str, updated_link: Link) -> Result<(), Box<dyn Error>> {
        if let Some(index) = self.links.iter().position(|link| link.title == title) {
            self.links[index] = updated_link;
            Ok(())
        } else {
            Err("Link not found".into())
        }
    }

    pub fn delete_link(&mut self, title: &str) -> Result<(), Box<dyn Error>> {
        if let Some(index) = self.links.iter().position(|link| link.title == title) {
            self.links.remove(index);
            Ok(())
        } else {
            Err("Link not found".into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_handler_operations() {
        let mut handler = LinkHandler::new();
        let link = Link::new(
            "Test Title".to_string(),
            Some("https://example.com".to_string()),
            vec!["Test Author".to_string()],
            None,
            NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            Some("Test Description".to_string()),
            vec!["test".to_string()],
        );

        handler.add_link(link);
        assert_eq!(handler.get_links().len(), 1);

        let found_link = handler.get_link_by_title("Test Title");
        assert!(found_link.is_some());

        let links_by_tag = handler.get_links_by_tag("test");
        assert_eq!(links_by_tag.len(), 1);
    }
}
