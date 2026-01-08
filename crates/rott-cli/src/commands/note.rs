//! Note command handlers
//!
//! Notes are children of links, providing annotations and comments.

use anyhow::{bail, Context, Result};
use uuid::Uuid;

use rott_core::{Note, Store};

use crate::editor::{confirm, edit_text};
use crate::output::Output;

/// Create a new note on a link
pub fn create(
    store: &mut Store,
    link_id: String,
    title: Option<String>,
    body: Option<String>,
    output: &Output,
) -> Result<()> {
    let link_uuid = parse_link_id(&link_id, store)?;

    // Get the link to show context
    let link = store
        .get_link(link_uuid)?
        .ok_or_else(|| anyhow::anyhow!("Link not found: {}", link_id))?;

    // Get body content
    let body_content = match body {
        Some(b) => b,
        None => {
            // Open editor for body
            let initial = format!(
                "<!-- Adding note to: {} -->\n<!-- {} -->\n\n",
                link.title, link.url
            );
            let edited = edit_text(&initial).context("Failed to edit note")?;

            // Remove the comment lines
            edited
                .lines()
                .filter(|line| !line.starts_with("<!--"))
                .collect::<Vec<_>>()
                .join("\n")
                .trim()
                .to_string()
        }
    };

    if body_content.is_empty() {
        bail!("Note body cannot be empty");
    }

    let note = match title {
        Some(t) => Note::with_title(t, body_content),
        None => Note::new(body_content),
    };

    let note_id = note.id;
    store
        .add_note_to_link(link_uuid, &note)
        .context("Failed to add note to link")?;

    output.success(&format!(
        "Added note {} to link {}",
        &note_id.to_string()[..8],
        &link_uuid.to_string()[..8]
    ));

    Ok(())
}

/// List all notes on a link
pub fn list(store: &Store, link_id: String, output: &Output) -> Result<()> {
    let link_uuid = parse_link_id(&link_id, store)?;

    let link = store
        .get_link(link_uuid)?
        .ok_or_else(|| anyhow::anyhow!("Link not found: {}", link_id))?;

    output.print_link_notes(&link);
    Ok(())
}

/// Delete a note from a link
pub fn delete(store: &mut Store, link_id: String, note_id: String, output: &Output) -> Result<()> {
    let link_uuid = parse_link_id(&link_id, store)?;

    let link = store
        .get_link(link_uuid)?
        .ok_or_else(|| anyhow::anyhow!("Link not found: {}", link_id))?;

    let note_uuid = parse_note_id(&note_id, &link)?;

    let note = link
        .get_note(note_uuid)
        .ok_or_else(|| anyhow::anyhow!("Note not found: {}", note_id))?;

    // Confirm deletion
    if output.should_prompt() {
        let preview = if note.body.len() > 50 {
            format!("{}...", &note.body[..50])
        } else {
            note.body.clone()
        };
        println!(
            "Delete note: {} - {}",
            &note.id.to_string()[..8],
            preview.replace('\n', " ")
        );
        if !confirm("Are you sure?")? {
            println!("Cancelled.");
            return Ok(());
        }
    }

    store
        .remove_note_from_link(link_uuid, note_uuid)
        .context("Failed to delete note")?;

    output.success(&format!("Deleted note: {}", &note_uuid.to_string()[..8]));

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

/// Parse a note ID (supports full UUID or prefix)
fn parse_note_id(id: &str, link: &rott_core::Link) -> Result<Uuid> {
    // Try full UUID first
    if let Ok(uuid) = Uuid::parse_str(id) {
        return Ok(uuid);
    }

    // Try prefix match
    let matches: Vec<_> = link
        .notes
        .iter()
        .filter(|n| n.id.to_string().starts_with(id))
        .collect();

    match matches.len() {
        0 => bail!("No note found matching: {}", id),
        1 => Ok(matches[0].id),
        _ => {
            eprintln!("Multiple notes match '{}':", id);
            for note in &matches {
                let preview = if note.body.len() > 30 {
                    format!("{}...", &note.body[..30])
                } else {
                    note.body.clone()
                };
                eprintln!("  {} - {}", &note.id.to_string()[..8], preview);
            }
            bail!("Ambiguous ID. Please provide more characters.");
        }
    }
}
