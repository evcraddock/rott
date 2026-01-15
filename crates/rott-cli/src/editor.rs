//! Interactive editing support
//!
//! Opens $EDITOR for editing note bodies and link metadata.

use anyhow::{bail, Context, Result};
use std::env;
use std::fs;
use std::io::{self, Write};
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
        bail!(
            "Editor '{}' exited with non-zero status. Check that your editor is configured correctly.",
            editor
        );
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

/// Prompt for confirmation
///
/// Returns true if user confirms, false otherwise.
/// In non-interactive mode (no TTY), returns false.
pub fn confirm(prompt: &str) -> Result<bool> {
    // Check if stdin is a TTY
    if !atty::is(atty::Stream::Stdin) {
        return Ok(false);
    }

    print!("{} [y/N] ", prompt);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let input = input.trim().to_lowercase();
    Ok(input == "y" || input == "yes")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_editor_with_env() {
        // This test depends on environment, so just verify it doesn't panic
        let _ = find_editor();
    }

    #[test]
    fn test_command_exists() {
        // "ls" should exist on Unix systems
        #[cfg(unix)]
        assert!(command_exists("ls"));

        // Random nonsense should not exist
        assert!(!command_exists("definitely_not_a_real_command_12345"));
    }
}
