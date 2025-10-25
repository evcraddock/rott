#[allow(dead_code)]
use std::fs;
use std::path::Path;

use super::{Link, LinkError};

pub struct LinkService;

impl LinkService {
    pub fn new() -> Self {
        Self
    }

    pub fn create_link(
        &self,
        url: String,
        tags: Vec<String>,
    ) -> Link {
        Link {
            title: url.clone(), // Use URL as title initially, will be replaced by metadata
            source: Some(url),
            author: Vec::new(),
            published: None,
            created: chrono::Local::now().date_naive(),
            description: None,
            tags,
            content: None,
            file_path: None,
        }
    }

    pub fn save_link_to_file(
        &self,
        link: &Link,
        directory_path: &str,
    ) -> Result<String, LinkError> {
        // Expand ~ to home directory
        let expanded_dir = if directory_path.starts_with("~/") {
            let home = dirs::home_dir()
                .ok_or_else(|| LinkError {
                    message: "Could not find home directory".to_string()
                })?;
            home.join(&directory_path[2..])
        } else {
            std::path::PathBuf::from(directory_path)
        };

        // Create directory if it doesn't exist
        fs::create_dir_all(&expanded_dir)?;

        // Generate filename from title (first 20 chars, sanitized)
        let sanitized_title = link.title
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '-' || c.is_whitespace())
            .take(20)
            .collect::<String>()
            .trim()
            .replace(' ', "-")
            .to_lowercase();

        let filename = if sanitized_title.is_empty() {
            format!("{}.md", chrono::Local::now().format("%Y-%m-%d-%H%M%S"))
        } else {
            format!("{}.md", sanitized_title)
        };
        let file_path = expanded_dir.join(&filename);

        // Serialize frontmatter
        let frontmatter = serde_yaml::to_string(&link)
            .map_err(|e| LinkError {
                message: format!("Failed to serialize frontmatter: {}", e)
            })?;

        // Create markdown content
        let content = format!("---\n{}---\n", frontmatter);

        // Write to file
        fs::write(&file_path, content)?;

        Ok(file_path.to_string_lossy().to_string())
    }

    pub fn delete_link(&self, file_path: &str) -> Result<(), LinkError> {
        match fs::remove_file(file_path) {
            Ok(_) => Ok(()),
            Err(e) => Err(LinkError::from(e)),
        }
    }

    pub fn update_tags(&self, file_path: &str, remove_tag: &str, add_tag: &str) -> Result<(), LinkError> {
        let content = fs::read_to_string(file_path)?;
        let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();

        let mut in_frontmatter = false;
        let mut in_tags_section = false;
        let mut tags_indent = String::new();
        let mut modified = false;

        for i in 0..lines.len() {
            let line = &lines[i];

            if line.trim() == "---" {
                if !in_frontmatter {
                    in_frontmatter = true;
                    continue;
                } else {
                    // End of frontmatter, add linkblog tag if we were in tags section
                    if in_tags_section && !modified {
                        lines.insert(i, format!("{}- {}", tags_indent, add_tag));
                    }
                    break;
                }
            }

            if in_frontmatter {
                if line.starts_with("tags:") {
                    in_tags_section = true;
                    continue;
                } else if in_tags_section {
                    if line.starts_with("- ") || line.trim().starts_with("- ") {
                        // Capture indentation from first tag
                        if tags_indent.is_empty() {
                            tags_indent = line.chars().take_while(|c| c.is_whitespace()).collect();
                        }

                        // Check if this line contains the tag to remove
                        let tag_value = line.trim().trim_start_matches("- ").trim();
                        if tag_value == remove_tag {
                            // Replace readlater with linkblog
                            lines[i] = format!("{}- {}", tags_indent, add_tag);
                            modified = true;
                            in_tags_section = false; // Stop processing tags
                            continue;
                        }
                    } else {
                        // No longer in tags section
                        if !modified {
                            // Add linkblog tag before this line
                            lines.insert(i, format!("{}- {}", tags_indent, add_tag));
                            modified = true;
                        }
                        in_tags_section = false;
                    }
                }
            }
        }

        let updated_content = lines.join("\n") + "\n";
        fs::write(file_path, updated_content)?;
        Ok(())
    }

    pub fn move_link(&self, file_path: &str, destination_dir: &str) -> Result<(), LinkError> {
        let source_path = Path::new(file_path);
        let file_name = source_path
            .file_name()
            .ok_or_else(|| LinkError {
                message: "Invalid file path".to_string()
            })?;

        // Expand ~ to home directory
        let expanded_dest = if destination_dir.starts_with("~/") {
            let home = dirs::home_dir()
                .ok_or_else(|| LinkError {
                    message: "Could not find home directory".to_string()
                })?;
            home.join(&destination_dir[2..])
        } else {
            Path::new(destination_dir).to_path_buf()
        };

        // Create destination directory if it doesn't exist
        fs::create_dir_all(&expanded_dest)?;

        let dest_path = expanded_dest.join(file_name);

        match fs::rename(source_path, &dest_path) {
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
            Err(e) => {
                eprintln!("YAML parse error: {:?}", e);
                eprintln!("Frontmatter content:\n{}", frontmatter_content);
                return Ok(Link::default())
            },
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
        
        // Always use file creation date instead of frontmatter date
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
        // The created date should come from the file creation time, not the frontmatter
        // Since we can't predict the exact file creation time in tests, we'll just check
        // that it's a valid date and not the default
        assert!(link.created != NaiveDate::from_ymd_opt(2025, 1, 1).unwrap());
        assert!(link.created <= chrono::Local::now().naive_local().date());
        assert_eq!(link.tags, vec!["test", "example"]);
    }

    #[test]
    fn test_create_link_basic() {
        let service = LinkService::new();
        let url = "https://example.com".to_string();
        let tags = vec!["test".to_string()];

        let link = service.create_link(url.clone(), tags.clone());

        assert_eq!(link.title, url);
        assert_eq!(link.source, Some(url));
        assert_eq!(link.tags, tags);
        assert!(link.author.is_empty());
        assert!(link.description.is_none());
        assert!(link.published.is_none());
        assert!(link.content.is_none());
        assert!(link.file_path.is_none());
        assert_eq!(link.created, chrono::Local::now().date_naive());
    }

    #[test]
    fn test_create_link_no_tags() {
        let service = LinkService::new();
        let url = "https://example.com".to_string();

        let link = service.create_link(url.clone(), Vec::new());

        assert_eq!(link.title, url);
        assert!(link.tags.is_empty());
    }

    #[test]
    fn test_save_link_to_file() {
        let dir = tempdir().unwrap();
        let service = LinkService::new();

        let link = service.create_link(
            "https://example.com".to_string(),
            vec!["test".to_string(), "example".to_string()],
        );

        let file_path = service
            .save_link_to_file(&link, dir.path().to_str().unwrap())
            .unwrap();

        // Verify file exists
        assert!(std::path::Path::new(&file_path).exists());

        // Verify file content
        let content = fs::read_to_string(&file_path).unwrap();
        assert!(content.starts_with("---\n"));
        assert!(content.contains("title:"));
        assert!(content.contains("source: https://example.com"));
        assert!(content.contains("test"));
        assert!(content.contains("example"));
        // Verify file_path and content are not in frontmatter
        assert!(!content.contains("file_path:"));
        assert!(!content.contains("content:"));
    }

    #[test]
    fn test_save_link_creates_directory() {
        let dir = tempdir().unwrap();
        let service = LinkService::new();

        // Create a nested path that doesn't exist
        let nested_path = dir.path().join("nested").join("directory");
        let link = service.create_link(
            "https://example.com".to_string(),
            vec!["test".to_string()],
        );

        let file_path = service
            .save_link_to_file(&link, nested_path.to_str().unwrap())
            .unwrap();

        // Verify directory was created
        assert!(nested_path.exists());
        assert!(std::path::Path::new(&file_path).exists());
    }

    #[test]
    fn test_save_link_filename_format() {
        let dir = tempdir().unwrap();
        let service = LinkService::new();

        // Create link with a descriptive title
        let mut link = service.create_link(
            "https://example.com".to_string(),
            Vec::new(),
        );
        link.title = "Rust Programming Guide".to_string();

        let file_path = service
            .save_link_to_file(&link, dir.path().to_str().unwrap())
            .unwrap();

        // Extract filename
        let filename = std::path::Path::new(&file_path)
            .file_name()
            .unwrap()
            .to_str()
            .unwrap();

        // Verify filename format: sanitized title (20 chars max)
        assert_eq!(filename, "rust-programming-gui.md");
    }

    #[test]
    fn test_save_link_filename_sanitization() {
        let dir = tempdir().unwrap();
        let service = LinkService::new();

        let mut link = service.create_link(
            "https://example.com".to_string(),
            Vec::new(),
        );
        link.title = "Test! Title: With (Punctuation)".to_string();

        let file_path = service
            .save_link_to_file(&link, dir.path().to_str().unwrap())
            .unwrap();

        let filename = std::path::Path::new(&file_path)
            .file_name()
            .unwrap()
            .to_str()
            .unwrap();

        // Should remove punctuation and keep only alphanumeric, dashes, and spaces (converted to dashes)
        assert_eq!(filename, "test-title-with-punc.md");
    }

    #[test]
    fn test_save_link_filename_fallback_empty_title() {
        let dir = tempdir().unwrap();
        let service = LinkService::new();

        let mut link = service.create_link(
            "https://example.com".to_string(),
            Vec::new(),
        );
        link.title = "!!!".to_string(); // Will be empty after sanitization

        let file_path = service
            .save_link_to_file(&link, dir.path().to_str().unwrap())
            .unwrap();

        let filename = std::path::Path::new(&file_path)
            .file_name()
            .unwrap()
            .to_str()
            .unwrap();

        // Should fallback to timestamp format
        assert!(filename.ends_with(".md"));
        assert!(filename.len() >= 20); // At least YYYY-MM-DD-HHMMSS.md
    }

    #[test]
    fn test_save_and_load_roundtrip() {
        let dir = tempdir().unwrap();
        let service = LinkService::new();

        let original_link = service.create_link(
            "https://example.com".to_string(),
            vec!["rust".to_string(), "programming".to_string()],
        );

        let file_path = service
            .save_link_to_file(&original_link, dir.path().to_str().unwrap())
            .unwrap();

        let loaded_link = service.load_from_file(&file_path).unwrap();

        assert_eq!(loaded_link.title, original_link.title);
        assert_eq!(loaded_link.source, original_link.source);
        assert_eq!(loaded_link.tags, original_link.tags);
        assert_eq!(loaded_link.author, original_link.author);
        assert_eq!(loaded_link.description, original_link.description);
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
