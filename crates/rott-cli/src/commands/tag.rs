//! Tag command handlers

use anyhow::Result;

use rott_core::Store;

use crate::output::Output;

/// List all tags with usage counts
pub fn list(store: &Store, output: &Output) -> Result<()> {
    let tags = store.get_tags_with_counts()?;
    output.print_tags(&tags);
    Ok(())
}
