//! Config command handlers

use std::path::PathBuf;

use anyhow::{bail, Context, Result};

use rott_core::Config;

use crate::output::{Output, OutputFormat};

/// Show current configuration
pub fn show(config_path: Option<&PathBuf>, output: &Output) -> Result<()> {
    let config =
        Config::load_with_cli_override(config_path).context("Failed to load configuration")?;

    match output.format {
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::json!({
                    "data_dir": config.data_dir,
                    "sync_url": config.sync_url,
                    "sync_enabled": config.sync_enabled,
                    "favorite_tag": config.favorite_tag,
                    "log_file": config.log_file
                })
            );
        }
        OutputFormat::Quiet => {
            println!("{}", config.data_dir.display());
        }
        OutputFormat::Human => {
            let effective_path = config_path
                .cloned()
                .unwrap_or_else(Config::config_file_path);
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
            println!(
                "  log_file:     {}",
                config
                    .log_file
                    .as_ref()
                    .map(|p| p.display().to_string())
                    .unwrap_or_else(|| "(not set)".to_string())
            );
            println!();
            println!("Config file: {}", effective_path.display());
        }
    }

    Ok(())
}

/// Set a configuration value
pub fn set(
    key: String,
    value: String,
    config_path: Option<&PathBuf>,
    output: &Output,
) -> Result<()> {
    let mut config =
        Config::load_with_cli_override(config_path).context("Failed to load configuration")?;

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
        "log_file" => {
            config.log_file = if value.is_empty() || value == "none" {
                None
            } else {
                Some(value.clone().into())
            };
        }
        _ => {
            bail!(
                "Unknown configuration key: '{}'\n\
                 Valid keys: data_dir, sync_url, sync_enabled, favorite_tag, log_file",
                key
            );
        }
    }

    // Save to the CLI-specified path or default
    let save_path = config_path
        .cloned()
        .unwrap_or_else(Config::config_file_path);
    config
        .save_to_path(&save_path)
        .context("Failed to save configuration")?;

    output.success(&format!("Set {} = {}", key, value));

    Ok(())
}
