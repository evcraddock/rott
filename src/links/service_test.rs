#[cfg(test)]
mod additional_tests {
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
}
