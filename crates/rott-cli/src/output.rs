//! Output formatting for CLI
//!
//! Provides consistent output formatting across all commands:
//! - Human-readable default output
//! - JSON output (--json flag)
//! - Quiet mode for scripting (--quiet flag)

use rott_core::{Link, Note};

/// Output format options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    /// Human-readable output (default)
    Human,
    /// JSON output
    Json,
    /// Quiet mode - minimal output
    Quiet,
}

impl OutputFormat {
    /// Create format from CLI flags
    pub fn from_flags(json: bool, quiet: bool) -> Self {
        if quiet {
            OutputFormat::Quiet
        } else if json {
            OutputFormat::Json
        } else {
            OutputFormat::Human
        }
    }
}

/// Output helper for consistent formatting
pub struct Output {
    /// The output format
    pub format: OutputFormat,
}

impl Output {
    pub fn new(format: OutputFormat) -> Self {
        Self { format }
    }

    /// Print a single link
    pub fn print_link(&self, link: &Link) {
        match self.format {
            OutputFormat::Human => {
                println!("ID:          {}", link.id);
                println!("Title:       {}", link.title);
                println!("URL:         {}", link.url);
                if let Some(ref desc) = link.description {
                    println!("Description: {}", desc);
                }
                if !link.author.is_empty() {
                    println!("Author:      {}", link.author.join(", "));
                }
                if !link.tags.is_empty() {
                    println!("Tags:        {}", link.tags.join(", "));
                }
                println!("Created:     {}", link.created_at.format("%Y-%m-%d %H:%M"));
                println!("Updated:     {}", link.updated_at.format("%Y-%m-%d %H:%M"));
            }
            OutputFormat::Json => {
                println!("{}", serde_json::to_string_pretty(link).unwrap());
            }
            OutputFormat::Quiet => {
                println!("{}", link.id);
            }
        }
    }

    /// Print a list of links
    pub fn print_links(&self, links: &[Link]) {
        match self.format {
            OutputFormat::Human => {
                if links.is_empty() {
                    println!("No links found.");
                    return;
                }
                for link in links {
                    println!(
                        "{} | {} | {}",
                        &link.id.to_string()[..8],
                        truncate(&link.title, 40),
                        truncate(&link.url, 50)
                    );
                }
                println!("\n{} link(s)", links.len());
            }
            OutputFormat::Json => {
                println!("{}", serde_json::to_string_pretty(links).unwrap());
            }
            OutputFormat::Quiet => {
                for link in links {
                    println!("{}", link.id);
                }
            }
        }
    }

    /// Print a single note
    pub fn print_note(&self, note: &Note) {
        match self.format {
            OutputFormat::Human => {
                println!("ID:      {}", note.id);
                println!("Title:   {}", note.title);
                if !note.tags.is_empty() {
                    println!("Tags:    {}", note.tags.join(", "));
                }
                println!("Created: {}", note.created_at.format("%Y-%m-%d %H:%M"));
                println!("Updated: {}", note.updated_at.format("%Y-%m-%d %H:%M"));
                println!();
                println!("{}", note.body);
            }
            OutputFormat::Json => {
                println!("{}", serde_json::to_string_pretty(note).unwrap());
            }
            OutputFormat::Quiet => {
                println!("{}", note.id);
            }
        }
    }

    /// Print a list of notes
    pub fn print_notes(&self, notes: &[Note]) {
        match self.format {
            OutputFormat::Human => {
                if notes.is_empty() {
                    println!("No notes found.");
                    return;
                }
                for note in notes {
                    println!(
                        "{} | {} | {}",
                        &note.id.to_string()[..8],
                        truncate(&note.title, 40),
                        note.created_at.format("%Y-%m-%d")
                    );
                }
                println!("\n{} note(s)", notes.len());
            }
            OutputFormat::Json => {
                println!("{}", serde_json::to_string_pretty(notes).unwrap());
            }
            OutputFormat::Quiet => {
                for note in notes {
                    println!("{}", note.id);
                }
            }
        }
    }

    /// Print a list of tags
    pub fn print_tags(&self, tags: &[(String, i64)]) {
        match self.format {
            OutputFormat::Human => {
                if tags.is_empty() {
                    println!("No tags found.");
                    return;
                }
                for (name, count) in tags {
                    println!("{} ({})", name, count);
                }
                println!("\n{} tag(s)", tags.len());
            }
            OutputFormat::Json => {
                let json_tags: Vec<_> = tags
                    .iter()
                    .map(|(name, count)| serde_json::json!({"name": name, "count": count}))
                    .collect();
                println!("{}", serde_json::to_string_pretty(&json_tags).unwrap());
            }
            OutputFormat::Quiet => {
                for (name, _) in tags {
                    println!("{}", name);
                }
            }
        }
    }

    /// Print a success message
    pub fn success(&self, message: &str) {
        match self.format {
            OutputFormat::Human => println!("✓ {}", message),
            OutputFormat::Json => {
                println!(
                    "{}",
                    serde_json::json!({"status": "success", "message": message})
                );
            }
            OutputFormat::Quiet => {}
        }
    }

    /// Check if we should prompt for confirmation
    pub fn should_prompt(&self) -> bool {
        self.format == OutputFormat::Human
    }
}

/// Truncate a string to max length, adding "..." if truncated
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_from_flags() {
        assert_eq!(OutputFormat::from_flags(false, false), OutputFormat::Human);
        assert_eq!(OutputFormat::from_flags(true, false), OutputFormat::Json);
        assert_eq!(OutputFormat::from_flags(false, true), OutputFormat::Quiet);
        // Quiet takes precedence
        assert_eq!(OutputFormat::from_flags(true, true), OutputFormat::Quiet);
    }

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("short", 10), "short");
        assert_eq!(truncate("this is a long string", 10), "this is...");
    }
}
