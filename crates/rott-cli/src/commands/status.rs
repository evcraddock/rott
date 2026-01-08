//! Status command handler

use anyhow::Result;

use rott_core::Store;

use crate::output::{Output, OutputFormat};

/// Show status information
pub fn show(store: &Store, output: &Output) -> Result<()> {
    let stats = store.storage_stats();
    let config = store.config();

    match output.format {
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::json!({
                    "root_id": store.root_id().to_bs58check(),
                    "root_url": store.root_url(),
                    "sync_enabled": config.sync_enabled,
                    "sync_url": config.sync_url,
                    "storage": {
                        "document_exists": stats.document_exists,
                        "database_exists": stats.database_exists,
                        "document_size": stats.document_size,
                        "database_size": stats.database_size,
                        "total_size": stats.total_size()
                    },
                    "counts": {
                        "links": store.link_count().unwrap_or(0),
                        "notes": store.note_count().unwrap_or(0)
                    }
                })
            );
        }
        OutputFormat::Quiet => {
            println!("{}", store.root_id());
        }
        OutputFormat::Human => {
            println!("ROTT Status");
            println!("===========");
            println!();
            println!("Root Document:");
            println!("  ID:  {}", store.root_id());
            println!("  URL: {}", store.root_url());
            println!();
            println!("Sync:");
            println!(
                "  Status: {}",
                if config.sync_enabled {
                    "enabled"
                } else {
                    "disabled"
                }
            );
            if let Some(ref url) = config.sync_url {
                println!("  Server: {}", url);
            }
            println!();
            println!("Storage:");
            println!("  Location: {}", config.data_dir.display());
            println!("  Size:     {}", stats.total_size_human());
            println!();
            println!("Contents:");
            println!("  Links: {}", store.link_count().unwrap_or(0));
            println!("  Notes: {}", store.note_count().unwrap_or(0));
        }
    }

    Ok(())
}
