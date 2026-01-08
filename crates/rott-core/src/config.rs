//! Application configuration
//!
//! Configuration is loaded from:
//! 1. Default values
//! 2. Config file (~/.config/rott/config.toml)
//! 3. Environment variables (ROTT_* prefix)
//!
//! Environment variables take precedence over config file values.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Environment variable prefix
const ENV_PREFIX: &str = "ROTT";

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Directory for data storage (Automerge doc, SQLite db)
    #[serde(default = "default_data_dir")]
    pub data_dir: PathBuf,

    /// Sync server URL (optional)
    #[serde(default)]
    pub sync_url: Option<String>,

    /// Whether sync is enabled
    #[serde(default)]
    pub sync_enabled: bool,

    /// Tag used for Favorites filter in TUI
    #[serde(default)]
    pub favorite_tag: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            data_dir: default_data_dir(),
            sync_url: None,
            sync_enabled: false,
            favorite_tag: None,
        }
    }
}

impl Config {
    /// Load configuration from default location and environment
    ///
    /// Order of precedence (highest to lowest):
    /// 1. Environment variables (ROTT_DATA_DIR, ROTT_SYNC_URL, ROTT_SYNC_ENABLED)
    /// 2. Config file (~/.config/rott/config.toml or ROTT_CONFIG)
    /// 3. Default values
    pub fn load() -> Result<Self> {
        Self::load_from_path(&Self::config_file_path())
    }

    /// Load configuration from a specific path
    ///
    /// Environment variables are still applied as overrides.
    /// If the file doesn't exist, defaults are used.
    pub fn load_from_path(path: &PathBuf) -> Result<Self> {
        let mut config = if path.exists() {
            let content = std::fs::read_to_string(path)
                .with_context(|| format!("Failed to read config file: {:?}", path))?;
            toml::from_str(&content)
                .with_context(|| format!("Failed to parse config file: {:?}", path))?
        } else {
            Self::default()
        };

        config.apply_env_overrides();
        config.ensure_data_dir()?;
        Ok(config)
    }

    /// Load configuration from a TOML string (useful for testing)
    pub fn load_from_str(toml_content: &str) -> Result<Self> {
        let mut config: Config =
            toml::from_str(toml_content).context("Failed to parse config TOML")?;
        config.apply_env_overrides();
        Ok(config)
    }

    /// Apply environment variable overrides
    fn apply_env_overrides(&mut self) {
        // ROTT_DATA_DIR
        if let Ok(val) = std::env::var(format!("{}_DATA_DIR", ENV_PREFIX)) {
            self.data_dir = PathBuf::from(val);
        }

        // ROTT_SYNC_URL
        if let Ok(val) = std::env::var(format!("{}_SYNC_URL", ENV_PREFIX)) {
            self.sync_url = if val.is_empty() { None } else { Some(val) };
        }

        // ROTT_SYNC_ENABLED
        if let Ok(val) = std::env::var(format!("{}_SYNC_ENABLED", ENV_PREFIX)) {
            self.sync_enabled = val.eq_ignore_ascii_case("true") || val == "1";
        }
    }

    /// Ensure data directory exists
    fn ensure_data_dir(&self) -> Result<()> {
        if !self.data_dir.exists() {
            std::fs::create_dir_all(&self.data_dir)
                .with_context(|| format!("Failed to create data directory: {:?}", self.data_dir))?;
        }
        Ok(())
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_file_path();

        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config directory: {:?}", parent))?;
        }

        let content = toml::to_string_pretty(self).context("Failed to serialize config")?;
        std::fs::write(&config_path, content)
            .with_context(|| format!("Failed to write config file: {:?}", config_path))?;
        Ok(())
    }

    /// Get the config file path
    ///
    /// Can be overridden with ROTT_CONFIG environment variable
    pub fn config_file_path() -> PathBuf {
        if let Ok(path) = std::env::var(format!("{}_CONFIG", ENV_PREFIX)) {
            return PathBuf::from(path);
        }

        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("rott")
            .join("config.toml")
    }

    /// Get the path to the Automerge document file
    pub fn automerge_path(&self) -> PathBuf {
        self.data_dir.join("document.automerge")
    }

    /// Get the path to the SQLite database
    pub fn sqlite_path(&self) -> PathBuf {
        self.data_dir.join("rott.db")
    }

    /// Get the path to the root document ID file
    pub fn root_doc_id_path(&self) -> PathBuf {
        self.data_dir.join("root_doc_id")
    }
}

/// Get the default data directory
fn default_data_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("rott")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::sync::Mutex;

    // Mutex to serialize tests that touch environment variables
    static ENV_MUTEX: Mutex<()> = Mutex::new(());

    /// Guard that locks env access and saves/restores env vars
    struct EnvGuard<'a> {
        _lock: std::sync::MutexGuard<'a, ()>,
        saved: Vec<(String, Option<String>)>,
    }

    impl<'a> EnvGuard<'a> {
        fn new(vars: &[&str]) -> Self {
            let lock = ENV_MUTEX.lock().unwrap();
            let saved = vars
                .iter()
                .map(|&name| (name.to_string(), env::var(name).ok()))
                .collect();
            // Clear all the vars
            for name in vars {
                env::remove_var(name);
            }
            Self { _lock: lock, saved }
        }
    }

    impl Drop for EnvGuard<'_> {
        fn drop(&mut self) {
            for (name, value) in &self.saved {
                match value {
                    Some(v) => env::set_var(name, v),
                    None => env::remove_var(name),
                }
            }
        }
    }

    const ENV_VARS: &[&str] = &["ROTT_DATA_DIR", "ROTT_SYNC_URL", "ROTT_SYNC_ENABLED"];

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(!config.sync_enabled);
        assert!(config.sync_url.is_none());
        assert!(config.data_dir.ends_with("rott"));
    }

    #[test]
    fn test_file_paths() {
        let config = Config::default();

        let am_path = config.automerge_path();
        assert!(am_path.ends_with("document.automerge"));

        let db_path = config.sqlite_path();
        assert!(db_path.ends_with("rott.db"));

        let id_path = config.root_doc_id_path();
        assert!(id_path.ends_with("root_doc_id"));
    }

    #[test]
    fn test_env_override_data_dir() {
        let _guard = EnvGuard::new(ENV_VARS);

        let mut config = Config::default();

        env::set_var("ROTT_DATA_DIR", "/tmp/rott-test");
        config.apply_env_overrides();

        assert_eq!(config.data_dir, PathBuf::from("/tmp/rott-test"));
    }

    #[test]
    fn test_env_override_sync_enabled() {
        let _guard = EnvGuard::new(ENV_VARS);

        let mut config = Config::default();
        assert!(!config.sync_enabled);

        env::set_var("ROTT_SYNC_ENABLED", "true");
        config.apply_env_overrides();
        assert!(config.sync_enabled);

        env::set_var("ROTT_SYNC_ENABLED", "1");
        config.sync_enabled = false;
        config.apply_env_overrides();
        assert!(config.sync_enabled);

        env::set_var("ROTT_SYNC_ENABLED", "false");
        config.apply_env_overrides();
        assert!(!config.sync_enabled);
    }

    #[test]
    fn test_env_override_sync_url() {
        let _guard = EnvGuard::new(ENV_VARS);

        let mut config = Config::default();
        assert!(config.sync_url.is_none());

        env::set_var("ROTT_SYNC_URL", "ws://localhost:3030");
        config.apply_env_overrides();
        assert_eq!(config.sync_url, Some("ws://localhost:3030".to_string()));

        // Empty string clears it
        env::set_var("ROTT_SYNC_URL", "");
        config.apply_env_overrides();
        assert!(config.sync_url.is_none());
    }

    #[test]
    fn test_serialization() {
        let _guard = EnvGuard::new(ENV_VARS);

        let config = Config {
            data_dir: PathBuf::from("/data/rott"),
            sync_url: Some("ws://sync.example.com".to_string()),
            sync_enabled: true,
            favorite_tag: None,
        };

        let toml_str = toml::to_string_pretty(&config).unwrap();
        assert!(toml_str.contains("data_dir"));
        assert!(toml_str.contains("sync_url"));
        assert!(toml_str.contains("sync_enabled"));

        let parsed: Config = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.data_dir, config.data_dir);
        assert_eq!(parsed.sync_url, config.sync_url);
        assert_eq!(parsed.sync_enabled, config.sync_enabled);
    }

    #[test]
    fn test_load_from_str() {
        let _guard = EnvGuard::new(ENV_VARS);

        let toml = r#"
            data_dir = "/custom/data"
            sync_url = "ws://example.com"
            sync_enabled = true
        "#;

        let config = Config::load_from_str(toml).unwrap();
        assert_eq!(config.data_dir, PathBuf::from("/custom/data"));
        assert_eq!(config.sync_url, Some("ws://example.com".to_string()));
        assert!(config.sync_enabled);
    }

    #[test]
    fn test_load_from_path_missing_file() {
        let _guard = EnvGuard::new(ENV_VARS);

        let path = PathBuf::from("/nonexistent/config.toml");
        let config = Config::load_from_path(&path).unwrap();
        // Should return defaults when file doesn't exist
        assert!(!config.sync_enabled);
        assert!(config.sync_url.is_none());
    }
}
