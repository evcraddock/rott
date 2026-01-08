//! Config command handlers

use anyhow::{bail, Context, Result};

use rott_core::Config;

use crate::output::{Output, OutputFormat};

/// Show current configuration
pub fn show(output: &Output) -> Result<()> {
    let config = Config::load().context("Failed to load configuration")?;

    match output.format {
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::json!({
                    "data_dir": config.data_dir,
                    "sync_url": config.sync_url,
                    "sync_enabled": config.sync_enabled,
                    "favorite_tag": config.favorite_tag
                })
            );
        }
        OutputFormat::Quiet => {
            println!("{}", config.data_dir.display());
        }
        OutputFormat::Human => {
            println!("Configuration:");
            println!("  data_dir:     {}", config.data_dir.display());
            println!(
                "  sync_url:     {}",
                config.sync_url.as_deref().unwrap_or("(not set)")
            );
            println!("  sync_enabled: {}", config.sync_enabled);
            println!(
                "  favorite_tag: {}",
                config.favorite_tag.as_deref().unwrap_or("(not set)")
            );
            println!();
            println!("Config file: {}", Config::config_file_path().display());
        }
    }

    Ok(())
}

/// Set a configuration value
pub fn set(key: String, value: String, output: &Output) -> Result<()> {
    let mut config = Config::load().context("Failed to load configuration")?;

    match key.as_str() {
        "data_dir" => {
            config.data_dir = value.clone().into();
        }
        "sync_url" => {
            config.sync_url = if value.is_empty() || value == "none" {
                None
            } else {
                Some(value.clone())
            };
        }
        "sync_enabled" => {
            config.sync_enabled = value
                .parse()
                .context("Invalid value for sync_enabled. Use 'true' or 'false'.")?;
        }
        "favorite_tag" => {
            config.favorite_tag = if value.is_empty() || value == "none" {
                None
            } else {
                Some(value.clone())
            };
        }
        _ => {
            bail!(
                "Unknown configuration key: '{}'\n\
                 Valid keys: data_dir, sync_url, sync_enabled, favorite_tag",
                key
            );
        }
    }

    config.save().context("Failed to save configuration")?;

    output.success(&format!("Set {} = {}", key, value));

    Ok(())
}
