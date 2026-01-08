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
    /// Manage links
    Link {
        #[command(subcommand)]
        command: LinkCommands,
    },
    /// Manage notes
    Note {
        #[command(subcommand)]
        command: NoteCommands,
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
    /// Show link details
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
}

#[derive(Subcommand)]
enum NoteCommands {
    /// Create a new note
    #[command(alias = "add")]
    Create {
        /// Note title
        title: String,
        /// Tags to add
        #[arg(short, long)]
        tag: Vec<String>,
        /// Note body (opens editor if not provided)
        #[arg(short, long)]
        body: Option<String>,
    },
    /// List all notes
    #[command(alias = "ls")]
    List {
        /// Filter by tag
        #[arg(short, long)]
        tag: Option<String>,
    },
    /// Show note details
    Show {
        /// Note ID (full UUID or prefix)
        id: String,
    },
    /// Edit a note
    Edit {
        /// Note ID (full UUID or prefix)
        id: String,
    },
    /// Delete a note
    #[command(alias = "rm")]
    Delete {
        /// Note ID (full UUID or prefix)
        id: String,
    },
    /// Search notes
    Search {
        /// Search query
        query: String,
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
        None => {
            println!("ROTT - Local-first links and notes management");
            println!();
            println!("Run 'rott --help' for usage information");
            return Ok(());
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
        }) | Some(Commands::Note {
            command: NoteCommands::Create { .. }
        }) | Some(Commands::Note {
            command: NoteCommands::Edit { .. }
        }) | Some(Commands::Note {
            command: NoteCommands::Delete { .. }
        })
    );

    let is_manual_sync = matches!(&cli.command, Some(Commands::Sync));

    // Sync before read commands (to get latest data)
    if !is_write && !is_manual_sync {
        auto_sync(&mut store, &output).await;
    }

    let result = match cli.command.unwrap() {
        Commands::Link { command } => handle_link_command(command, &mut store, &output).await,
        Commands::Note { command } => handle_note_command(command, &mut store, &output),
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
    }
}

fn handle_note_command(command: NoteCommands, store: &mut Store, output: &Output) -> Result<()> {
    match command {
        NoteCommands::Create { title, tag, body } => {
            commands::note::create(store, title, tag, body, output)
        }
        NoteCommands::List { tag } => commands::note::list(store, tag, output),
        NoteCommands::Show { id } => commands::note::show(store, id, output),
        NoteCommands::Edit { id } => commands::note::edit(store, id, output),
        NoteCommands::Delete { id } => commands::note::delete(store, id, output),
        NoteCommands::Search { query } => commands::note::search(store, query, output),
    }
}

fn handle_config_command(command: Option<ConfigCommands>, output: &Output) -> Result<()> {
    match command {
        Some(ConfigCommands::Show) | None => commands::config::show(output),
        Some(ConfigCommands::Set { key, value }) => commands::config::set(key, value, output),
    }
}
