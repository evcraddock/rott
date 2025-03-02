#[allow(dead_code)]
use std::fs;
use std::path::Path;

use super::{Link, LinkError};

pub struct LinkService;

impl LinkService {
    pub fn new() -> Self {
        Self
    }

    pub fn delete_link(&self, file_path: &str) -> Result<(), LinkError> {
        match fs::remove_file(file_path) {
            Ok(_) => Ok(()),
            Err(e) => Err(LinkError::from(e)),
        }
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
        let content = fs::read_to_string(&file_path)?;
        let mut link = self.frontmatter_to_link(&content)?;
        link.file_path = Some(file_path.as_ref().to_string_lossy().to_string());
        
        // Get file metadata and set created timestamp
        if let Ok(metadata) = fs::metadata(&file_path) {
            if let Ok(created) = metadata.created() {
                if let Ok(datetime) = created.duration_since(std::time::UNIX_EPOCH) {
                    let naive_datetime = {
                        let secs = datetime.as_secs() as i64;
                        let nsecs = datetime.subsec_nanos();
                        chrono::DateTime::from_timestamp(secs, nsecs).map(|dt| dt.naive_utc())
                    };
                    if let Some(datetime) = naive_datetime {
                        link.created = datetime.date();
                    }
                }
            }
        }
        
        Ok(link)
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
    fn test_delete_link() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("to_delete.md");
        
        // Create a test file
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"Test content").unwrap();
        
        let service = LinkService::new();
        
        // Verify the file exists
        assert!(file_path.exists());
        
        // Delete the file
        let result = service.delete_link(file_path.to_str().unwrap());
        
        // Verify the operation succeeded
        assert!(result.is_ok());
        
        // Verify the file no longer exists
        assert!(!file_path.exists());
    }

    #[test]
    fn test_delete_nonexistent_link() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("nonexistent.md");
        
        let service = LinkService::new();
        
        // Try to delete a file that doesn't exist
        let result = service.delete_link(file_path.to_str().unwrap());
        
        // Verify the operation failed
        assert!(result.is_err());
    }

    #[test]
    fn test_malformed_frontmatter() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("malformed.md");
        
        // Create a file with malformed YAML
        let content = r#"---
title: "Malformed
tags: [unclosed
---
Content"#;
        
        let mut file = File::create(&file_path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
        
        let service = LinkService::new();
        let link = service.load_from_file(&file_path).unwrap();
        
        // Should fall back to default Link when YAML parsing fails
        assert_eq!(link.title, "");
        assert!(link.file_path.is_some());
    }

    #[test]
    fn test_empty_directory() {
        let dir = tempdir().unwrap();
        
        let service = LinkService::new();
        let links = service.load_from_directory(dir.path()).unwrap();
        
        // Should return an empty vector for an empty directory
        assert!(links.is_empty());
    }

    #[test]
    fn test_no_frontmatter() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("no_frontmatter.md");
        
        // Create a file with no frontmatter
        let content = "Just some content without frontmatter";
        
        let mut file = File::create(&file_path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
        
        let service = LinkService::new();
        let link = service.load_from_file(&file_path).unwrap();
        
        // Should return a default Link
        assert_eq!(link.title, "");
        assert!(link.file_path.is_some());
    }

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
        // The created date comes from the frontmatter in our test file
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
