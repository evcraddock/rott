use std::error::Error;

use super::Link;

pub struct LinkHandler {
    links: Vec<Link>,
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
