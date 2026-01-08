//! ROTT CLI
//!
//! Command-line interface for ROTT - links and notes management.

use anyhow::Result;
use clap::{Parser, Subcommand};

use rott_core::{Config, DocumentId, Identity, Store};

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
    /// Initialize ROTT (first-time setup)
    Init {
        /// Create a new identity (skip interactive prompt)
        #[arg(long, conflicts_with = "join")]
        new: bool,
        /// Join an existing identity by providing root document ID
        #[arg(long, conflicts_with = "new")]
        join: Option<String>,
    },
    /// Device identity management
    Device {
        #[command(subcommand)]
        command: Option<DeviceCommands>,
    },
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

#[derive(Subcommand, Clone)]
enum DeviceCommands {
    /// Show root document ID
    Show,
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

    // Commands that don't need initialization or the store
    match &cli.command {
        Some(Commands::Config { command }) => {
            return handle_config_command(command.clone(), &output);
        }
        Some(Commands::Init { new, join }) => {
            return handle_init_command(*new, join.clone(), &output);
        }
        _ => {}
    }

    // Check if initialized - if not, run first-time setup
    let identity = Identity::new()?;
    if !identity.is_initialized() {
        // For TUI, we'll handle setup there
        if matches!(&cli.command, Some(Commands::Tui) | None) {
            // TUI will handle its own setup flow
        } else {
            // For CLI commands, run interactive setup first
            run_first_time_setup(&output)?;
        }
    }

    // Handle TUI (default when no command given)
    if matches!(&cli.command, Some(Commands::Tui) | None) {
        return tui::run().await;
    }

    // Handle device command (doesn't need full store)
    if let Some(Commands::Device { command }) = &cli.command {
        return handle_device_command(command.clone(), &output);
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
        Commands::Tui => unreachable!(),           // Handled above
        Commands::Init { .. } => unreachable!(),   // Handled above
        Commands::Device { .. } => unreachable!(), // Handled above
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

fn handle_init_command(new: bool, join: Option<String>, output: &Output) -> Result<()> {
    let identity = Identity::new()?;

    if identity.is_initialized() {
        let root_id = identity.root_id()?.unwrap();
        println!();
        println!("Already initialized.");
        println!("Root document ID: {}", root_id);
        println!();
        println!("To start fresh, remove:");
        println!("  {}", identity.data_dir().display());
        return Ok(());
    }

    if let Some(id_str) = join {
        // --join <id>: Join existing identity (no prompt)
        let root_id = DocumentId::from_bs58check(&id_str)
            .map_err(|e| anyhow::anyhow!("Invalid root document ID: {}", e))?;

        let result = identity.initialize_join(root_id)?;

        if output.is_json() {
            println!(
                "{}",
                serde_json::json!({
                    "root_id": result.root_id.to_bs58check(),
                    "is_new": false
                })
            );
        } else if !output.is_quiet() {
            println!();
            println!("Identity configured.");
            println!();
            let config = Config::load()?;
            if config.sync_url.is_none() {
                println!("Sync server not configured. Your data will sync once you set one:");
                println!("  rott config set sync_url ws://your-server:3030");
            }
        }
    } else if new {
        // --new: Create new identity (no prompt)
        let result = identity.initialize_new()?;

        if output.is_json() {
            println!(
                "{}",
                serde_json::json!({
                    "root_id": result.root_id.to_bs58check(),
                    "is_new": true
                })
            );
        } else if !output.is_quiet() {
            println!();
            println!("Created new identity.");
            println!();
            println!("Your root document ID: {}", result.root_id);
            println!();
            println!("This ID is stored in: {}", identity.data_dir().display());
            println!("View it anytime with: rott device show");
        } else {
            // Quiet mode - just print the ID
            println!("{}", result.root_id);
        }
    } else {
        // No flags: Interactive prompt
        run_first_time_setup(output)?;
    }

    Ok(())
}

fn handle_device_command(command: Option<DeviceCommands>, output: &Output) -> Result<()> {
    let identity = Identity::new()?;

    if !identity.is_initialized() {
        anyhow::bail!("Not initialized. Run `rott init` first.");
    }

    let root_id = identity.root_id()?.unwrap();

    match command {
        Some(DeviceCommands::Show) | None => {
            if output.is_json() {
                println!(
                    "{}",
                    serde_json::json!({
                        "root_id": root_id.to_bs58check(),
                        "root_url": root_id.to_url()
                    })
                );
            } else if output.is_quiet() {
                println!("{}", root_id);
            } else {
                println!();
                println!("Root document ID: {}", root_id);
                println!("Automerge URL:    {}", root_id.to_url());
                println!();
                println!("Use this ID to set up ROTT on another device:");
                println!("  rott init --join {}", root_id);
            }
        }
    }

    Ok(())
}

/// Run first-time setup interactively
fn run_first_time_setup(_output: &Output) -> Result<()> {
    use std::io::{self, Write};

    println!();
    println!("Welcome to ROTT!");
    println!();
    println!("No existing identity found. Is this your first device?");
    println!();
    println!("  [1] Yes, create new identity");
    println!("  [2] No, I have an existing root document ID");
    println!();
    print!("> ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let choice = input.trim();

    match choice {
        "1" => {
            let identity = Identity::new()?;
            let result = identity.initialize_new()?;

            println!();
            println!("Created new identity.");
            println!();
            println!("Your root document ID: {}", result.root_id);
            println!();
            println!("This ID is stored in: {}", identity.data_dir().display());
            println!("View it anytime with: rott device show");
            println!();
        }
        "2" => {
            print!("Enter your root document ID: ");
            io::stdout().flush()?;

            let mut id_input = String::new();
            io::stdin().read_line(&mut id_input)?;
            let id_str = id_input.trim();

            let root_id = DocumentId::from_bs58check(id_str)
                .map_err(|e| anyhow::anyhow!("Invalid root document ID: {}", e))?;

            let identity = Identity::new()?;
            identity.initialize_join(root_id)?;

            println!();
            println!("Identity configured.");
            println!();
            let config = Config::load()?;
            if config.sync_url.is_none() {
                println!("Sync server not configured. Your data will sync once you set one:");
                println!("  rott config set sync_url ws://your-server:3030");
            }
            println!();
        }
        _ => {
            anyhow::bail!("Invalid choice. Please run the command again and enter 1 or 2.");
        }
    }

    Ok(())
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
