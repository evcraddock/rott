//! Note command handlers

use anyhow::{bail, Context, Result};
use uuid::Uuid;

use rott_core::{Note, Store};

use crate::editor::{confirm, edit_text};
use crate::output::Output;

/// Create a new note
pub fn create(
    store: &mut Store,
    title: String,
    tags: Vec<String>,
    body: Option<String>,
    output: &Output,
) -> Result<()> {
    let mut note = Note::new(&title);

    // Add tags
    for tag in tags {
        note.add_tag(tag);
    }

    // Get body content
    let body_content = match body {
        Some(b) => b,
        None => {
            // Open editor for body
            let initial = format!("# {}\n\n<!-- Enter your note content below -->\n\n", title);
            edit_text(&initial).context("Failed to edit note")?
        }
    };

    note.set_body(body_content);

    store.add_note(&note).context("Failed to create note")?;

    output.success(&format!("Created note: {}", note.id));
    output.print_note(&note);

    Ok(())
}

/// List all notes, optionally filtered by tag
pub fn list(store: &Store, tag: Option<String>, output: &Output) -> Result<()> {
    let notes = match tag {
        Some(ref t) => store.get_notes_by_tag(t)?,
        None => store.get_all_notes()?,
    };

    output.print_notes(&notes);
    Ok(())
}

/// Show a single note
pub fn show(store: &Store, id: String, output: &Output) -> Result<()> {
    let uuid = parse_note_id(&id, store)?;

    let note = store
        .get_note(uuid)?
        .ok_or_else(|| anyhow::anyhow!("Note not found: {}", id))?;

    output.print_note(&note);
    Ok(())
}

/// Edit a note
pub fn edit(store: &mut Store, id: String, output: &Output) -> Result<()> {
    let uuid = parse_note_id(&id, store)?;

    let mut note = store
        .get_note(uuid)?
        .ok_or_else(|| anyhow::anyhow!("Note not found: {}", id))?;

    // Create editable content
    let content = format!(
        "title: {}\ntags: {}\n---\n{}",
        note.title,
        note.tags.join(", "),
        note.body
    );

    let edited = edit_text(&content).context("Failed to edit note")?;

    // Parse edited content
    let (new_title, new_tags, new_body) = parse_note_content(&edited)?;

    note.set_title(new_title);
    note.set_tags(new_tags);
    note.set_body(new_body);

    store.update_note(&note).context("Failed to update note")?;

    output.success("Note updated");
    output.print_note(&note);

    Ok(())
}

/// Delete a note
pub fn delete(store: &mut Store, id: String, output: &Output) -> Result<()> {
    let uuid = parse_note_id(&id, store)?;

    let note = store
        .get_note(uuid)?
        .ok_or_else(|| anyhow::anyhow!("Note not found: {}", id))?;

    // Confirm deletion
    if output.should_prompt() {
        println!(
            "Delete note: {} - {}",
            &note.id.to_string()[..8],
            note.title
        );
        if !confirm("Are you sure?")? {
            println!("Cancelled.");
            return Ok(());
        }
    }

    store.delete_note(uuid).context("Failed to delete note")?;

    output.success(&format!("Deleted note: {}", uuid));

    Ok(())
}

/// Search notes
pub fn search(store: &Store, query: String, output: &Output) -> Result<()> {
    let notes = store.search_notes(&query)?;
    output.print_notes(&notes);
    Ok(())
}

/// Parse a note ID (supports full UUID or prefix)
fn parse_note_id(id: &str, store: &Store) -> Result<Uuid> {
    // Try full UUID first
    if let Ok(uuid) = Uuid::parse_str(id) {
        return Ok(uuid);
    }

    // Try prefix match
    let notes = store.get_all_notes()?;
    let matches: Vec<_> = notes
        .iter()
        .filter(|n| n.id.to_string().starts_with(id))
        .collect();

    match matches.len() {
        0 => bail!("No note found matching: {}", id),
        1 => Ok(matches[0].id),
        _ => {
            eprintln!("Multiple notes match '{}':", id);
            for note in &matches {
                eprintln!("  {} - {}", note.id, note.title);
            }
            bail!("Ambiguous ID. Please provide more characters.");
        }
    }
}

/// Parse note content from editor format
fn parse_note_content(content: &str) -> Result<(String, Vec<String>, String)> {
    let mut lines = content.lines();

    // Parse title line
    let title_line = lines.next().unwrap_or("");
    let title = if title_line.starts_with("title:") {
        title_line.trim_start_matches("title:").trim().to_string()
    } else {
        title_line.trim().to_string()
    };

    if title.is_empty() {
        bail!("Note title cannot be empty");
    }

    // Parse tags line
    let tags_line = lines.next().unwrap_or("");
    let tags: Vec<String> = if tags_line.starts_with("tags:") {
        tags_line
            .trim_start_matches("tags:")
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    } else {
        Vec::new()
    };

    // Skip separator line (---)
    let next_line = lines.next().unwrap_or("");
    let body_start = if next_line.trim() == "---" {
        lines.collect::<Vec<_>>().join("\n")
    } else {
        // No separator, include this line in body
        std::iter::once(next_line)
            .chain(lines)
            .collect::<Vec<_>>()
            .join("\n")
    };

    Ok((title, tags, body_start.trim().to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_note_content() {
        let content = "title: My Note\ntags: rust, programming\n---\nThis is the body.";
        let (title, tags, body) = parse_note_content(content).unwrap();

        assert_eq!(title, "My Note");
        assert_eq!(tags, vec!["rust", "programming"]);
        assert_eq!(body, "This is the body.");
    }

    #[test]
    fn test_parse_note_content_no_tags() {
        let content = "title: My Note\ntags: \n---\nBody content here.";
        let (title, tags, body) = parse_note_content(content).unwrap();

        assert_eq!(title, "My Note");
        assert!(tags.is_empty());
        assert_eq!(body, "Body content here.");
    }

    #[test]
    fn test_parse_note_content_multiline_body() {
        let content = "title: Test\ntags: test\n---\nLine 1\nLine 2\nLine 3";
        let (_, _, body) = parse_note_content(content).unwrap();

        assert_eq!(body, "Line 1\nLine 2\nLine 3");
    }
}
