//! ROTT CLI
//!
//! Command-line interface for ROTT - links and notes management.

use anyhow::Result;
use clap::{Parser, Subcommand};

use rott_core::{Config, Store};

mod commands;
mod editor;
mod metadata;
mod output;
mod tui;

use output::{Output, OutputFormat};

#[derive(Parser)]
#[command(name = "rott")]
#[command(about = "ROTT - Local-first links and notes management")]
#[command(version)]
#[command(propagate_version = true)]
struct Cli {
    /// Output as JSON
    #[arg(long, global = true)]
    json: bool,

    /// Quiet mode - minimal output
    #[arg(short, long, global = true)]
    quiet: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the TUI interface
    Tui,
    /// Manage links
    Link {
        #[command(subcommand)]
        command: LinkCommands,
    },
    /// List all tags
    Tags,
    /// Show or set configuration
    Config {
        #[command(subcommand)]
        command: Option<ConfigCommands>,
    },
    /// Show status (root doc ID, sync status)
    Status,
    /// Sync with remote server
    Sync,
}

#[derive(Subcommand)]
enum LinkCommands {
    /// Create a new link
    #[command(alias = "add")]
    Create {
        /// URL to save
        url: String,
        /// Tags to add
        #[arg(short, long)]
        tag: Vec<String>,
    },
    /// List all links
    #[command(alias = "ls")]
    List {
        /// Filter by tag
        #[arg(short, long)]
        tag: Option<String>,
    },
    /// Show link details (including notes)
    Show {
        /// Link ID (full UUID or prefix)
        id: String,
    },
    /// Edit a link
    Edit {
        /// Link ID (full UUID or prefix)
        id: String,
    },
    /// Delete a link
    #[command(alias = "rm")]
    Delete {
        /// Link ID (full UUID or prefix)
        id: String,
    },
    /// Search links
    Search {
        /// Search query
        query: String,
    },
    /// Manage notes on a link
    Note {
        #[command(subcommand)]
        command: NoteCommands,
    },
}

#[derive(Subcommand)]
enum NoteCommands {
    /// Add a note to a link
    #[command(alias = "add")]
    Create {
        /// Link ID (full UUID or prefix)
        link_id: String,
        /// Note title (optional)
        #[arg(short = 'T', long)]
        title: Option<String>,
        /// Note body (opens editor if not provided)
        #[arg(short, long)]
        body: Option<String>,
    },
    /// List notes on a link
    #[command(alias = "ls")]
    List {
        /// Link ID (full UUID or prefix)
        link_id: String,
    },
    /// Delete a note from a link
    #[command(alias = "rm")]
    Delete {
        /// Link ID (full UUID or prefix)
        link_id: String,
        /// Note ID (full UUID or prefix)
        note_id: String,
    },
}

#[derive(Subcommand, Clone)]
enum ConfigCommands {
    /// Show current configuration
    Show,
    /// Set a configuration value
    Set {
        /// Configuration key (data_dir, sync_url, sync_enabled)
        key: String,
        /// Configuration value
        value: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let output = Output::new(OutputFormat::from_flags(cli.json, cli.quiet));

    // Commands that don't need the store
    match &cli.command {
        Some(Commands::Config { command }) => {
            return handle_config_command(command.clone(), &output);
        }
        Some(Commands::Tui) | None => {
            // Launch TUI (default when no command given)
            return tui::run().await;
        }
        _ => {}
    }

    // Open store for commands that need it
    let mut store = Store::open()?;

    // Determine if this is a read or write command
    let is_write = matches!(
        &cli.command,
        Some(Commands::Link {
            command: LinkCommands::Create { .. }
        }) | Some(Commands::Link {
            command: LinkCommands::Edit { .. }
        }) | Some(Commands::Link {
            command: LinkCommands::Delete { .. }
        }) | Some(Commands::Link {
            command: LinkCommands::Note {
                command: NoteCommands::Create { .. }
            }
        }) | Some(Commands::Link {
            command: LinkCommands::Note {
                command: NoteCommands::Delete { .. }
            }
        })
    );

    let is_manual_sync = matches!(&cli.command, Some(Commands::Sync));

    // Sync before read commands (to get latest data)
    if !is_write && !is_manual_sync {
        auto_sync(&mut store, &output).await;
    }

    let result = match cli.command.unwrap() {
        Commands::Tui => unreachable!(), // Handled above
        Commands::Link { command } => handle_link_command(command, &mut store, &output).await,
        Commands::Tags => commands::tag::list(&store, &output),
        Commands::Config { .. } => unreachable!(), // Handled above
        Commands::Status => commands::status::show(&store, &output),
        Commands::Sync => commands::sync::sync(&mut store, &output).await,
    };

    // Sync after write commands (to push changes)
    if is_write {
        auto_sync(&mut store, &output).await;
    }

    result
}

async fn handle_link_command(
    command: LinkCommands,
    store: &mut Store,
    output: &Output,
) -> Result<()> {
    match command {
        LinkCommands::Create { url, tag } => commands::link::create(store, url, tag, output).await,
        LinkCommands::List { tag } => commands::link::list(store, tag, output),
        LinkCommands::Show { id } => commands::link::show(store, id, output),
        LinkCommands::Edit { id } => commands::link::edit(store, id, output),
        LinkCommands::Delete { id } => commands::link::delete(store, id, output),
        LinkCommands::Search { query } => commands::link::search(store, query, output),
        LinkCommands::Note { command } => handle_note_command(command, store, output),
    }
}

fn handle_note_command(command: NoteCommands, store: &mut Store, output: &Output) -> Result<()> {
    match command {
        NoteCommands::Create {
            link_id,
            title,
            body,
        } => commands::note::create(store, link_id, title, body, output),
        NoteCommands::List { link_id } => commands::note::list(store, link_id, output),
        NoteCommands::Delete { link_id, note_id } => {
            commands::note::delete(store, link_id, note_id, output)
        }
    }
}

fn handle_config_command(command: Option<ConfigCommands>, output: &Output) -> Result<()> {
    match command {
        Some(ConfigCommands::Show) | None => commands::config::show(output),
        Some(ConfigCommands::Set { key, value }) => commands::config::set(key, value, output),
    }
}

/// Auto-sync if sync is enabled, silently handles errors
async fn auto_sync(store: &mut Store, output: &Output) {
    let config = match Config::load() {
        Ok(c) => c,
        Err(_) => return,
    };

    if !config.sync_enabled || config.sync_url.is_none() {
        return;
    }

    // Sync silently (errors shown only in non-quiet mode)
    if let Err(e) = commands::sync::sync_quiet(store, &config).await {
        if !output.is_quiet() {
            eprintln!("⚠ Auto-sync failed: {}", e);
        }
    }
}
