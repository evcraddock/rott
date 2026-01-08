//! Link command handlers

use anyhow::{bail, Context, Result};
use uuid::Uuid;

use rott_core::{Link, Store};

use crate::editor::confirm;
use crate::metadata::fetch_metadata;
use crate::output::Output;

/// Create a new link
pub async fn create(store: &mut Store, url: String, tags: Vec<String>, output: &Output) -> Result<()> {
    // Fetch metadata from URL
    let metadata = fetch_metadata(&url).await;

    let mut link = Link::new(&url);

    // Apply fetched metadata
    if let Some(title) = metadata.title {
        link.set_title(title);
    }
    if let Some(desc) = metadata.description {
        link.set_description(Some(desc));
    }
    if !metadata.author.is_empty() {
        link.set_author(metadata.author);
    }

    // Add tags
    for tag in tags {
        link.add_tag(tag);
    }

    store.add_link(&link).context("Failed to create link")?;

    output.success(&format!("Created link: {}", link.id));
    output.print_link(&link);

    Ok(())
}

/// List all links, optionally filtered by tag
pub fn list(store: &Store, tag: Option<String>, output: &Output) -> Result<()> {
    let links = match tag {
        Some(ref t) => store.get_links_by_tag(t)?,
        None => store.get_all_links()?,
    };

    output.print_links(&links);
    Ok(())
}

/// Show a single link
pub fn show(store: &Store, id: String, output: &Output) -> Result<()> {
    let uuid = parse_link_id(&id, store)?;

    let link = store
        .get_link(uuid)?
        .ok_or_else(|| anyhow::anyhow!("Link not found: {}", id))?;

    output.print_link(&link);
    Ok(())
}

/// Edit a link
pub fn edit(store: &mut Store, id: String, output: &Output) -> Result<()> {
    let uuid = parse_link_id(&id, store)?;

    let mut link = store
        .get_link(uuid)?
        .ok_or_else(|| anyhow::anyhow!("Link not found: {}", id))?;

    // Interactive editing
    println!("Editing link: {}", link.id);
    println!("Press Enter to keep current value, or type new value.\n");

    // Title
    let current_title = &link.title;
    if let Some(new_title) = prompt_with_default("Title", current_title)? {
        link.set_title(new_title);
    }

    // Description
    let current_desc = link.description.as_deref().unwrap_or("");
    if let Some(new_desc) = prompt_with_default("Description", current_desc)? {
        link.set_description(if new_desc.is_empty() {
            None
        } else {
            Some(new_desc)
        });
    }

    // Tags
    let current_tags = link.tags.join(", ");
    println!(
        "Current tags: {}",
        if current_tags.is_empty() {
            "(none)"
        } else {
            &current_tags
        }
    );
    if let Some(new_tags) = prompt_optional("New tags (comma-separated)")? {
        let tags: Vec<String> = new_tags
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        link.set_tags(tags);
    }

    store.update_link(&link).context("Failed to update link")?;

    output.success("Link updated");
    output.print_link(&link);

    Ok(())
}

/// Delete a link
pub fn delete(store: &mut Store, id: String, output: &Output) -> Result<()> {
    let uuid = parse_link_id(&id, store)?;

    let link = store
        .get_link(uuid)?
        .ok_or_else(|| anyhow::anyhow!("Link not found: {}", id))?;

    // Confirm deletion
    if output.should_prompt() {
        println!(
            "Delete link: {} - {}",
            &link.id.to_string()[..8],
            link.title
        );
        if !confirm("Are you sure?")? {
            println!("Cancelled.");
            return Ok(());
        }
    }

    store.delete_link(uuid).context("Failed to delete link")?;

    output.success(&format!("Deleted link: {}", uuid));

    Ok(())
}

/// Search links
pub fn search(store: &Store, query: String, output: &Output) -> Result<()> {
    let links = store.search_links(&query)?;
    output.print_links(&links);
    Ok(())
}

/// Parse a link ID (supports full UUID or prefix)
fn parse_link_id(id: &str, store: &Store) -> Result<Uuid> {
    // Try full UUID first
    if let Ok(uuid) = Uuid::parse_str(id) {
        return Ok(uuid);
    }

    // Try prefix match
    let links = store.get_all_links()?;
    let matches: Vec<_> = links
        .iter()
        .filter(|l| l.id.to_string().starts_with(id))
        .collect();

    match matches.len() {
        0 => bail!("No link found matching: {}", id),
        1 => Ok(matches[0].id),
        _ => {
            eprintln!("Multiple links match '{}':", id);
            for link in &matches {
                eprintln!("  {} - {}", link.id, link.title);
            }
            bail!("Ambiguous ID. Please provide more characters.");
        }
    }
}

/// Prompt with a default value, returns None if user keeps default
fn prompt_with_default(prompt: &str, default: &str) -> Result<Option<String>> {
    use std::io::{self, Write};

    if default.is_empty() {
        print!("{}: ", prompt);
    } else {
        print!("{} [{}]: ", prompt, default);
    }
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim();

    if input.is_empty() {
        Ok(None)
    } else {
        Ok(Some(input.to_string()))
    }
}

/// Prompt for optional value
fn prompt_optional(prompt: &str) -> Result<Option<String>> {
    use std::io::{self, Write};

    print!("{}: ", prompt);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim();

    if input.is_empty() {
        Ok(None)
    } else {
        Ok(Some(input.to_string()))
    }
}
