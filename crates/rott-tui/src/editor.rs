//! Interactive editing support
//!
//! Opens $EDITOR for editing note bodies and link metadata.

use anyhow::{bail, Context, Result};
use std::env;
use std::fs;
use std::process::Command;

/// Open content in the user's preferred editor
///
/// Uses $EDITOR, $VISUAL, or falls back to common editors.
pub fn edit_text(initial_content: &str) -> Result<String> {
    let editor = find_editor()?;

    // Create temp file with content
    let temp_dir = env::temp_dir();
    let temp_path = temp_dir.join(format!("rott_edit_{}.md", std::process::id()));

    fs::write(&temp_path, initial_content)
        .with_context(|| format!("Failed to create temp file: {:?}", temp_path))?;

    // Open editor
    let status = Command::new(&editor)
        .arg(&temp_path)
        .status()
        .with_context(|| format!("Failed to run editor: {}", editor))?;

    if !status.success() {
        // Clean up temp file
        let _ = fs::remove_file(&temp_path);
        bail!("Editor exited with non-zero status");
    }

    // Read edited content
    let content = fs::read_to_string(&temp_path)
        .with_context(|| format!("Failed to read edited file: {:?}", temp_path))?;

    // Clean up
    let _ = fs::remove_file(&temp_path);

    Ok(content)
}

/// Find the user's preferred editor
fn find_editor() -> Result<String> {
    // Check environment variables
    if let Ok(editor) = env::var("EDITOR") {
        if !editor.is_empty() {
            return Ok(editor);
        }
    }

    if let Ok(visual) = env::var("VISUAL") {
        if !visual.is_empty() {
            return Ok(visual);
        }
    }

    // Try common editors
    let common_editors = ["nano", "vim", "vi", "emacs", "code", "notepad"];

    for editor in common_editors {
        if command_exists(editor) {
            return Ok(editor.to_string());
        }
    }

    bail!(
        "No editor found. Set $EDITOR environment variable.\n\
         Example: export EDITOR=nano"
    )
}

/// Check if a command exists in PATH
fn command_exists(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}
