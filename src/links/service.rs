#[allow(dead_code)]
use std::fs;
use std::path::Path;

use super::{Link, LinkError};

pub struct LinkService;

impl LinkService {
    pub fn new() -> Self {
        Self
    }

    pub fn load_from_directory<P: AsRef<Path>>(
        &self,
        directory_path: P,
    ) -> Result<Vec<Link>, LinkError> {
        let mut links = Vec::new();
        let paths = fs::read_dir(directory_path)?;

        for path in paths {
            let path = path?.path();
            if path.extension().map_or(false, |ext| ext == "md") {
                match self.load_from_file(&path) {
                    Ok(link) => links.push(link),
                    Err(e) => eprintln!("Error loading file {:?}: {:?}", path, e),
                }
            }
        }

        Ok(links)
    }

    fn frontmatter_to_link(&self, frontmatter: &str) -> Result<Link, LinkError> {
        let mut lines = frontmatter.lines().peekable();
        let mut frontmatter_content = String::new();
        let mut content = String::new();
        let mut in_frontmatter = false;
        let mut past_frontmatter = false;

        // Extract frontmatter content between --- markers
        while let Some(line) = lines.next() {
            if line.trim() == "---" {
                if !in_frontmatter {
                    in_frontmatter = true;
                    continue;
                } else {
                    past_frontmatter = true;
                    continue;
                }
            }

            if past_frontmatter {
                content.push_str(line);
                content.push('\n');
            } else if in_frontmatter {
                frontmatter_content.push_str(line);
                frontmatter_content.push('\n');
            }
        }

        let mut link: Link = match serde_yaml::from_str(&frontmatter_content) {
            Ok(link) => link,
            Err(_) => return Ok(Link::default()),
        };

        link.content = if content.trim().is_empty() {
            None
        } else {
            Some(content)
        };

        Ok(link)
    }

    pub fn load_from_file<P: AsRef<Path>>(&self, file_path: P) -> Result<Link, LinkError> {
        let content = fs::read_to_string(file_path)?;
        self.frontmatter_to_link(&content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_load_markdown_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.md");
        let mut file = File::create(&file_path).unwrap();

        let content = r#"---
title: Test Document
source: https://example.com
author:
  - John Doe
  - Jane Smith
created: 2025-01-01
description: A test document
tags:
  - test
  - example
---
Test content"#;

        file.write_all(content.as_bytes()).unwrap();

        let service = LinkService::new();
        let link = service.load_from_file(file_path).unwrap();

        assert_eq!(link.title, "Test Document");
        assert_eq!(link.source, Some("https://example.com".to_string()));
        assert_eq!(link.author, vec!["John Doe", "Jane Smith"]);
        assert_eq!(link.created, NaiveDate::from_ymd_opt(2025, 1, 1).unwrap());
        assert_eq!(link.tags, vec!["test", "example"]);
    }

    #[test]
    fn test_load_from_directory() {
        let dir = tempdir().unwrap();

        // Create test files
        let files = vec![
            (
                "test1.md",
                r#"---
title: Test 1
created: 2025-01-01
---"#,
            ),
            (
                "test2.md",
                r#"---
title: Test 2
created: 2025-01-02
---"#,
            ),
            ("not_a_markdown.txt", "Just a text file"),
        ];

        for (filename, content) in files {
            let file_path = dir.path().join(filename);
            let mut file = File::create(file_path).unwrap();
            file.write_all(content.as_bytes()).unwrap();
        }

        let service = LinkService::new();
        let links = service.load_from_directory(dir.path()).unwrap();

        assert_eq!(links.len(), 2); // Should only load the .md files
    }
}
