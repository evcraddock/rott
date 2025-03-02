#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::tempdir;

    #[test]
    fn test_load_config_from_file() {
        // Create a temporary directory for our test config
        let dir = tempdir().unwrap();
        let config_dir = dir.path().join(".config").join("rott");
        fs::create_dir_all(&config_dir).unwrap();
        
        // Create a test config file
        let config_path = config_dir.join("config.yaml");
        let config_content = r#"
links_path: "/test/path"
default_topic: "test-topic"
"#;
        fs::write(&config_path, config_content).unwrap();
        
        // Temporarily set HOME to our test directory
        let original_home = env::var("HOME").ok();
        env::set_var("HOME", dir.path().to_str().unwrap());
        
        // Load the config
        let config = load_config().unwrap();
        
        // Restore original HOME
        if let Some(home) = original_home {
            env::set_var("HOME", home);
        } else {
            env::remove_var("HOME");
        }
        
        // Verify the config was loaded correctly
        assert_eq!(config.links_path, "/test/path");
        assert_eq!(config.default_topic, "test-topic");
    }

    #[test]
    fn test_load_config_from_env() {
        // Set environment variables
        env::set_var("APP_LINKS_PATH", "/env/test/path");
        env::set_var("APP_DEFAULT_TOPIC", "env-test-topic");
        
        // Load the config
        let config = load_config().unwrap();
        
        // Clean up
        env::remove_var("APP_LINKS_PATH");
        env::remove_var("APP_DEFAULT_TOPIC");
        
        // Verify environment variables were used
        assert_eq!(config.links_path, "/env/test/path");
        assert_eq!(config.default_topic, "env-test-topic");
    }

    #[test]
    fn test_config_env_overrides_file() {
        // Create a temporary directory for our test config
        let dir = tempdir().unwrap();
        let config_dir = dir.path().join(".config").join("rott");
        fs::create_dir_all(&config_dir).unwrap();
        
        // Create a test config file
        let config_path = config_dir.join("config.yaml");
        let config_content = r#"
links_path: "/file/path"
default_topic: "file-topic"
"#;
        fs::write(&config_path, config_content).unwrap();
        
        // Set environment variables that should override the file
        env::set_var("APP_LINKS_PATH", "/override/path");
        
        // Temporarily set HOME to our test directory
        let original_home = env::var("HOME").ok();
        env::set_var("HOME", dir.path().to_str().unwrap());
        
        // Load the config
        let config = load_config().unwrap();
        
        // Restore original HOME and clean up env vars
        if let Some(home) = original_home {
            env::set_var("HOME", home);
        } else {
            env::remove_var("HOME");
        }
        env::remove_var("APP_LINKS_PATH");
        
        // Verify the environment variable overrode the file setting
        assert_eq!(config.links_path, "/override/path");
        assert_eq!(config.default_topic, "file-topic");
    }
}
