#![allow(dead_code)]
mod app;
mod cli;
mod config;
mod links;
mod metadata;

use app::{ui, ActivePane, App};
use cli::{Cli, Commands, CreateCommands};
use clap::Parser;
use config::load_config;
use crossterm::{
    event::{self, KeyCode, KeyEventKind, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    prelude::*,
    widgets::{Block, Paragraph},
};

use std::{
    io::{self, stdout},
    sync::Arc,
};

fn main_debug() {
    let config = Arc::new(load_config().expect("could not load config"));
    App::new(None, &config);
}

fn handle_create_link(url: String, tags: Option<Vec<String>>, config: &Arc<config::AppConfig>) {
    use links::LinkService;

    let service = LinkService::new();
    let tag_vec = tags.unwrap_or_default();

    // Create basic link
    let mut link = service.create_link(url.clone(), tag_vec.clone());

    // Try to fetch metadata
    println!("Fetching metadata from {}...", url);
    match metadata::fetch_url_metadata(&url) {
        Ok(page_metadata) => {
            if let Some(title) = page_metadata.title {
                link.title = title.clone();
                println!("  Title: {}", title);
            }
            if let Some(description) = page_metadata.description {
                link.description = Some(description.clone());
                println!("  Description: {}", if description.len() > 80 {
                    format!("{}...", &description[..80])
                } else {
                    description
                });
            }
            if !page_metadata.author.is_empty() {
                link.author = page_metadata.author.clone();
                println!("  Author: {}", page_metadata.author.join(", "));
            }
        }
        Err(e) => {
            eprintln!("  Warning: Could not fetch metadata: {}", e);
            eprintln!("  Continuing with URL as title...");
        }
    }

    // Save to file
    match service.save_link_to_file(&link, &config.links_path) {
        Ok(file_path) => {
            println!("\n✓ Link created successfully!");
            if !tag_vec.is_empty() {
                println!("  Tags: {}", tag_vec.join(", "));
            }
            println!("  File: {}", file_path);
        }
        Err(e) => {
            eprintln!("\n✗ Error creating link: {}", e);
            std::process::exit(1);
        }
    }
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();
    let config = Arc::new(load_config().expect("could not load config"));

    // Handle CLI commands
    match cli.command {
        Some(Commands::Create { resource }) => {
            match resource {
                CreateCommands::Link { url, tags } => {
                    handle_create_link(url, tags, &config);
                    return Ok(());
                }
            }
        }
        None => {
            // No command specified, launch TUI
        }
    }

    // Setup terminal
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    // Create app state
    let mut app = App::new(Some(config.default_topic.clone()), &config);
    app.topics.state.select(Some(0));

    loop {
        terminal.draw(|frame| {
            let size = frame.area();

            // Draw main UI
            ui(frame, &mut app);
            // Draw legend at bottom
            let legend = vec![
                "q: Quit",
                "Tab/l: Right pane",
                "Shift+Tab/h: Left pane",
                "↑/k: Move up",
                "↓/j: Move down",
                "Enter: Open link",
                "Del: Remove link",
                "Shift+S: Move to drafts",
            ];
            let legend_text = Paragraph::new(legend.join(" | "))
                .block(Block::default())
                .alignment(Alignment::Center);

            let legend_area = Rect::new(0, size.height - 1, size.width, 1);
            frame.render_widget(legend_text, legend_area);
        })?;

        // Handle input
        if event::poll(std::time::Duration::from_millis(16))? {
            if let event::Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Tab | KeyCode::Char('l') => {
                            if app.active_pane == ActivePane::Topics {
                                app.active_pane = ActivePane::Pages;
                                app.pages.state.select(Some(0));
                            }
                        }
                        KeyCode::BackTab | KeyCode::Char('h') => {
                            if app.active_pane == ActivePane::Pages {
                                app.active_pane = ActivePane::Topics;
                                app.pages.state.select(None);
                            }
                        }
                        KeyCode::Down | KeyCode::Char('j') => match app.active_pane {
                            ActivePane::Topics => {
                                app.topics.next();
                                if let Some(index) = app.topics.state.selected() {
                                    let selected_topic = app.topics.items[index].clone();
                                    app.reload(selected_topic);
                                }
                            }
                            ActivePane::Pages => app.pages.next(),
                        },
                        KeyCode::Up | KeyCode::Char('k') => match app.active_pane {
                            ActivePane::Topics => {
                                app.topics.previous();
                                if let Some(index) = app.topics.state.selected() {
                                    let selected_topic = app.topics.items[index].clone();
                                    app.reload(selected_topic);
                                }
                            }
                            ActivePane::Pages => app.pages.previous(),
                        },
                        KeyCode::Char('r') => {
                            if app.active_pane == ActivePane::Pages {
                                if let Some(index) = app.topics.state.selected() {
                                    let selected_topic = app.topics.items[index].clone();
                                    app.reload(selected_topic);
                                } else if !app.topics.items.is_empty() {
                                    app.topics.state.select(Some(0));
                                    let selected_topic = app.topics.items[0].clone();
                                    app.reload(selected_topic);
                                }
                            } else if app.active_pane == ActivePane::Topics {
                                app = App::new(None, &config);
                                app.topics.state.select(Some(0));
                            }
                        }
                        KeyCode::Enter => {
                            if app.active_pane == ActivePane::Pages {
                                if let Some(index) = app.pages.state.selected() {
                                    if let Some(url) = app.pages.items[index].source.clone() {
                                        let _ = open::that(url);
                                    }
                                }
                            }
                        }
                        KeyCode::Delete => {
                            if app.active_pane == ActivePane::Pages {
                                if let Some(index) = app.pages.state.selected() {
                                    app.delete_link(&app.pages.items[index].clone());
                                    if let Some(topic_index) = app.topics.state.selected() {
                                        let selected_topic = app.topics.items[topic_index].clone();
                                        app.reload(selected_topic);
                                        let total_items = app.pages.items.len() - 1;
                                        app.pages.state.select(Some(if index <= total_items {
                                            index
                                        } else {
                                            index - 1
                                        }));
                                    }
                                }
                            }
                        }
                        KeyCode::Char('S') if key.modifiers.contains(KeyModifiers::SHIFT) => {
                            if app.active_pane == ActivePane::Pages {
                                if let Some(index) = app.pages.state.selected() {
                                    app.move_link_to_drafts(&app.pages.items[index].clone());
                                    if let Some(topic_index) = app.topics.state.selected() {
                                        let selected_topic = app.topics.items[topic_index].clone();
                                        app.reload(selected_topic);
                                        let total_items = app.pages.items.len() - 1;
                                        app.pages.state.select(Some(if index <= total_items {
                                            index
                                        } else {
                                            index - 1
                                        }));
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}
