#![allow(dead_code)]
mod app;
mod cli;
mod config;
mod links;
mod metadata;
mod watcher;

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

fn handle_create_link(url: String, tags: Option<Vec<String>>, silent: bool, config: &Arc<config::AppConfig>) {
    use links::LinkService;

    let service = LinkService::new();
    let tag_vec = tags.unwrap_or_default();

    // Create basic link
    let mut link = service.create_link(url.clone(), tag_vec.clone());

    // Try to fetch metadata
    if !silent {
        println!("Fetching metadata from {}...", url);
    }
    match metadata::fetch_url_metadata(&url) {
        Ok(page_metadata) => {
            if let Some(title) = page_metadata.title {
                link.title = title.clone();
                if !silent {
                    println!("  Title: {}", title);
                }
            }
            if let Some(description) = page_metadata.description {
                link.description = Some(description.clone());
                if !silent {
                    println!("  Description: {}", if description.len() > 80 {
                        format!("{}...", &description[..80])
                    } else {
                        description
                    });
                }
            }
            if !page_metadata.author.is_empty() {
                link.author = page_metadata.author.clone();
                if !silent {
                    println!("  Author: {}", page_metadata.author.join(", "));
                }
            }
        }
        Err(e) => {
            if !silent {
                eprintln!("  Warning: Could not fetch metadata: {}", e);
                eprintln!("  Continuing with URL as title...");
            }
        }
    }

    // Save to file
    match service.save_link_to_file(&link, &config.links_path) {
        Ok(file_path) => {
            if !silent {
                println!("\n✓ Link created successfully!");
                if !tag_vec.is_empty() {
                    println!("  Tags: {}", tag_vec.join(", "));
                }
                println!("  File: {}", file_path);
            }
        }
        Err(e) => {
            if !silent {
                eprintln!("\n✗ Error creating link: {}", e);
            }
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
                CreateCommands::Link { url, tags, silent } => {
                    handle_create_link(url, tags, silent, &config);
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
    if !app.topics.items.is_empty() {
        app.topics.state.select(Some(0));
    }

    // Setup file watcher for the links directory
    let file_watcher = match watcher::FileWatcher::new(&config.links_path) {
        Ok(watcher) => Some(watcher),
        Err(e) => {
            eprintln!("Warning: Could not setup file watcher: {}", e);
            eprintln!("Auto-refresh will not work. You can manually refresh with 'r'.");
            None
        }
    };

    loop {
        terminal.draw(|frame| {
            let size = frame.area();

            // Draw main UI
            ui(frame, &mut app);
            // Draw legend at bottom
            let legend = if app.input_mode == app::InputMode::EditingTags {
                vec![
                    "Enter: Save",
                    "Esc: Cancel",
                ]
            } else {
                vec![
                    "q: Quit",
                    "Tab/l: Right",
                    "Shift+Tab/h: Left",
                    "↑/k: Up",
                    "↓/j: Down",
                    "Enter: Open",
                    "Del: Remove",
                    "Shift+S: Drafts",
                    "Shift+T: Edit tags",
                ]
            };
            let legend_text = Paragraph::new(legend.join(" | "))
                .block(Block::default())
                .alignment(Alignment::Center);

            let legend_area = Rect::new(0, size.height - 1, size.width, 1);
            frame.render_widget(legend_text, legend_area);
        })?;

        // Check for file system changes
        if let Some(ref watcher) = file_watcher {
            if watcher.check_events() {
                // Files changed, reload current topic
                if let Some(topic_index) = app.topics.state.selected() {
                    let selected_topic = app.topics.items[topic_index].clone();
                    app.reload(selected_topic);
                }
            }
        }

        // Handle input
        if event::poll(std::time::Duration::from_millis(16))? {
            if let event::Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    // Handle input mode-specific keys
                    if app.input_mode == app::InputMode::EditingTags {
                        match key.code {
                            KeyCode::Enter => {
                                app.save_edited_tags();
                                // Reload to show updated tags
                                if let Some(topic_index) = app.topics.state.selected() {
                                    let selected_topic = app.topics.items[topic_index].clone();
                                    app.reload(selected_topic);
                                }
                            }
                            KeyCode::Esc => {
                                app.cancel_editing();
                            }
                            KeyCode::Char(c) => {
                                app.tag_input.push(c);
                            }
                            KeyCode::Backspace => {
                                app.tag_input.pop();
                            }
                            _ => {}
                        }
                        continue;
                    }

                    // Normal mode key handling
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
                                if !app.topics.items.is_empty() {
                                    app.topics.state.select(Some(0));
                                }
                            }
                        }
                        KeyCode::Enter => {
                            if app.active_pane == ActivePane::Pages {
                                if let Some(index) = app.pages.state.selected() {
                                    if index < app.pages.items.len() {
                                        if let Some(url) = app.pages.items[index].source.clone() {
                                            let _ = open::that(url);
                                        }
                                    }
                                }
                            }
                        }
                        KeyCode::Delete => {
                            if app.active_pane == ActivePane::Pages {
                                if let Some(index) = app.pages.state.selected() {
                                    if index < app.pages.items.len() {
                                        app.delete_link(&app.pages.items[index].clone());
                                        if let Some(topic_index) = app.topics.state.selected() {
                                            let selected_topic = app.topics.items[topic_index].clone();

                                            // Calculate which index to select after deletion
                                            let new_index = if index == 0 {
                                                // Deleted first item, select new first item (index 0)
                                                Some(0)
                                            } else {
                                                // Try to keep same index, or go to last item if we're past the end
                                                Some(index)
                                            };

                                            // Reload with the calculated selection
                                            app.reload_with_page_selection(selected_topic, new_index);
                                        }
                                    }
                                }
                            }
                        }
                        KeyCode::Char('S') if key.modifiers.contains(KeyModifiers::SHIFT) => {
                            if app.active_pane == ActivePane::Pages {
                                if let Some(index) = app.pages.state.selected() {
                                    if index < app.pages.items.len() {
                                        app.move_link_to_drafts(&app.pages.items[index].clone());
                                        if let Some(topic_index) = app.topics.state.selected() {
                                            let selected_topic = app.topics.items[topic_index].clone();

                                            // Calculate which index to select after moving
                                            let new_index = if index == 0 {
                                                // Moved first item, select new first item (index 0)
                                                Some(0)
                                            } else {
                                                // Try to keep same index, or go to last item if we're past the end
                                                Some(index)
                                            };

                                            // Reload with the calculated selection
                                            app.reload_with_page_selection(selected_topic, new_index);
                                        }
                                    }
                                }
                            }
                        }
                        KeyCode::Char('T') if key.modifiers.contains(KeyModifiers::SHIFT) => {
                            if app.active_pane == ActivePane::Pages {
                                app.start_editing_tags();
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
