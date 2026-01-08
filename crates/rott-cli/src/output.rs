//! Output formatting for CLI
//!
//! Provides consistent output formatting across all commands:
//! - Human-readable default output
//! - JSON output (--json flag)
//! - Quiet mode for scripting (--quiet flag)

use rott_core::Link;

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

    /// Check if output is in quiet mode
    pub fn is_quiet(&self) -> bool {
        matches!(self.format, OutputFormat::Quiet)
    }

    /// Print a single link (with notes summary)
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

                // Show notes
                if !link.notes.is_empty() {
                    println!();
                    println!("── Notes ({}) ──", link.notes.len());
                    for note in &link.notes {
                        let preview = truncate_line(&note.body, 60);
                        if let Some(ref title) = note.title {
                            println!(
                                "[{}] {} - {}",
                                note.created_at.format("%Y-%m-%d"),
                                title,
                                preview
                            );
                        } else {
                            println!("[{}] {}", note.created_at.format("%Y-%m-%d"), preview);
                        }
                    }
                }
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
                    let notes_indicator = if link.notes.is_empty() {
                        String::new()
                    } else {
                        format!(" [{}]", link.notes.len())
                    };
                    println!(
                        "{} | {}{} | {}",
                        &link.id.to_string()[..8],
                        truncate(&link.title, 35),
                        notes_indicator,
                        truncate(&link.url, 45)
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

    /// Print notes for a specific link
    pub fn print_link_notes(&self, link: &Link) {
        match self.format {
            OutputFormat::Human => {
                println!("Notes for: {} - {}", &link.id.to_string()[..8], link.title);
                println!();

                if link.notes.is_empty() {
                    println!("No notes on this link.");
                    return;
                }

                for note in &link.notes {
                    println!("────────────────────────────────────────");
                    println!(
                        "ID: {}  Created: {}",
                        &note.id.to_string()[..8],
                        note.created_at.format("%Y-%m-%d %H:%M")
                    );
                    if let Some(ref title) = note.title {
                        println!("Title: {}", title);
                    }
                    println!();
                    println!("{}", note.body);
                    println!();
                }
                println!("{} note(s)", link.notes.len());
            }
            OutputFormat::Json => {
                println!("{}", serde_json::to_string_pretty(&link.notes).unwrap());
            }
            OutputFormat::Quiet => {
                for note in &link.notes {
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

    /// Print an informational message
    pub fn message(&self, msg: &str) {
        match self.format {
            OutputFormat::Human => println!("{}", msg),
            OutputFormat::Json => {
                println!("{}", serde_json::json!({"message": msg}));
            }
            OutputFormat::Quiet => {}
        }
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

/// Truncate to first line and max length
fn truncate_line(s: &str, max_len: usize) -> String {
    let first_line = s.lines().next().unwrap_or("");
    truncate(first_line, max_len)
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

    #[test]
    fn test_truncate_line() {
        assert_eq!(truncate_line("single line", 20), "single line");
        assert_eq!(truncate_line("line one\nline two", 20), "line one");
        assert_eq!(
            truncate_line("very long single line here", 10),
            "very lo..."
        );
    }
}
