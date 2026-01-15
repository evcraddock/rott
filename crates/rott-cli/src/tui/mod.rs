//! ROTT TUI
//!
//! Terminal user interface for ROTT - links and notes management.
//!
//! ## Layout
//!
//! Three-pane layout:
//! - Left: Filters (Favorites, Recent, Untagged, By Tag...)
//! - Middle: Items list (links)
//! - Right: Detail preview (selected link details and notes)
//!
//! ## Navigation
//!
//! - j/k or ↑/↓: Move selection up/down
//! - h/l or ←/→: Switch focus between panes
//! - Tab: Cycle through panes
//! - Enter: Select filter / Open link in browser
//! - q: Quit
//!
//! ## Commands
//!
//! - a: Add link
//! - t: Edit tags
//! - n: Add note
//! - e: Edit link
//! - d: Delete link
//! - u: Undo delete
//! - /: Filter current view
//! - :: Command mode

mod app;
mod sync;
mod ui;

use std::fs::File;

use anyhow::Result;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::prelude::*;
use rott_core::{Config, Store};
use std::io::stdout;
use tracing::info;
use tracing_subscriber::EnvFilter;

use app::{App, CommandResult, CommandType, EditorTask, InputMode, SyncIndicator};
use rott_core::sync::{PersistentSyncHandle, SyncCommand, SyncTaskEvent};

use crate::editor;
use crate::metadata;

/// Run the TUI application
pub async fn run() -> Result<()> {
    // Open the store
    let mut store = Store::open()?;
    let config = store.config().clone();

    // Initialize TUI logging (file-based, only if ROTT_LOG is set)
    init_tui_logging(&config);

    // Setup terminal
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    // Create app
    let mut app = App::new(&store)?;

    // Start sync if enabled
    let sync_handle = if sync::is_sync_enabled(&config) {
        app.sync_status = SyncIndicator::Syncing;
        terminal.draw(|frame| ui::draw(frame, &app))?;

        // Spawn persistent sync task (maintains WebSocket connection)
        sync::spawn_persistent_sync(&store, &config)
    } else {
        None
    };

    // Apply initial filter (Favorites)
    app.apply_filter(&store)?;

    // Run app
    let result = run_app(&mut terminal, &mut app, &mut store, sync_handle).await;

    // Restore terminal
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;

    result
}

async fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    store: &mut Store,
    mut sync_handle: Option<PersistentSyncHandle>,
) -> Result<()> {
    // Track if we need to push changes after this iteration
    let mut pending_push = false;

    loop {
        // Check for status message timeout
        app.check_status_timeout();

        // Draw UI
        terminal.draw(|frame| ui::draw(frame, app))?;

        // Handle events with a short timeout
        tokio::select! {
            biased;

            // Check for sync events (if sync is enabled)
            sync_event = async {
                if let Some(ref mut handle) = sync_handle {
                    handle.event_rx.recv().await
                } else {
                    // Never resolves if no sync handle
                    std::future::pending::<Option<SyncTaskEvent>>().await
                }
            } => {
                if let Some(event) = sync_event {
                    match event {
                        SyncTaskEvent::StatusChanged(status) => {
                            app.sync_status = sync::status_to_indicator(status);
                        }
                        SyncTaskEvent::DocumentUpdated => {
                            // Remote changes received - rebuild projection and refresh UI
                            if let Err(e) = store.rebuild_projection() {
                                app.set_error(format!("Failed to rebuild after sync: {}", e));
                            } else if let Err(e) = app.refresh(store) {
                                app.set_error(format!("Failed to refresh after sync: {}", e));
                            } else {
                                app.set_status("Synced remote changes".to_string());
                            }
                        }
                        SyncTaskEvent::Error(msg) => {
                            app.set_status(format!("Sync error: {}", msg));
                            app.sync_status = SyncIndicator::Error;
                        }
                    }
                }
            }

            // Poll for terminal events
            _ = tokio::time::sleep(std::time::Duration::from_millis(50)) => {
                // Push changes if needed
                if pending_push {
                    pending_push = false;
                    if let Some(ref handle) = sync_handle {
                        // Signal sync task to push local changes
                        let _ = handle.command_tx.send(SyncCommand::PushChanges).await;
                    }
                }

                // Check for terminal events (non-blocking)
                if event::poll(std::time::Duration::from_millis(0))? {
                    if let Event::Key(key) = event::read()? {
                        // Only handle key press events (not release)
                        if key.kind != KeyEventKind::Press {
                            continue;
                        }

                        // If error modal is showing, any key dismisses it
                        if app.has_error() {
                            app.clear_error();
                            continue;
                        }

                        // If help is showing, any key dismisses it
                        if app.show_help {
                            app.show_help = false;
                            continue;
                        }

                        // Handle based on input mode
                        match app.input_mode {
                            InputMode::Normal => {
                                if let Some(needs_push) = handle_normal_mode(app, store, key.code, key.modifiers).await? {
                                    if needs_push {
                                        pending_push = true;
                                    }
                                }
                            }
                            InputMode::Command => {
                                if let Some(needs_push) = handle_command_mode(terminal, app, store, key.code, key.modifiers).await? {
                                    if needs_push {
                                        pending_push = true;
                                    }
                                }
                            }
                            InputMode::Filter => {
                                if let Err(e) = handle_filter_mode(app, store, key.code) {
                                    app.set_error(format!("Filter error: {}", e));
                                }
                            }
                        }
                    }
                }
            }
        }

        if app.should_quit {
            // Shutdown sync task
            if let Some(handle) = sync_handle.take() {
                let _ = handle.command_tx.send(SyncCommand::Shutdown).await;
            }
            break;
        }
    }

    Ok(())
}

/// Handle key events in normal mode
/// Returns Some(true) if local changes need to be pushed, Some(false) if not, None for no action
async fn handle_normal_mode(
    app: &mut App,
    store: &mut Store,
    code: KeyCode,
    modifiers: KeyModifiers,
) -> Result<Option<bool>> {
    // Clear status message on navigation keys
    match code {
        KeyCode::Char('j')
        | KeyCode::Char('k')
        | KeyCode::Up
        | KeyCode::Down
        | KeyCode::Char('h')
        | KeyCode::Char('l')
        | KeyCode::Left
        | KeyCode::Right
        | KeyCode::Tab
        | KeyCode::BackTab
        | KeyCode::Char('g')
        | KeyCode::Char('G') => {
            app.status_message = None;
        }
        _ => {}
    }

    // Clear pending 'g' if timeout expired (500ms)
    if let Some(time) = app.pending_g {
        if time.elapsed() > std::time::Duration::from_millis(500) {
            app.pending_g = None;
        }
    }

    match code {
        // Quit
        KeyCode::Char('q') => {
            app.should_quit = true;
        }
        KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => {
            app.should_quit = true;
        }

        // Navigation: up
        KeyCode::Char('k') | KeyCode::Up => {
            app.move_up();
        }

        // Navigation: down
        KeyCode::Char('j') | KeyCode::Down => {
            app.move_down();
        }

        // Navigation: left pane
        KeyCode::Char('h') | KeyCode::Left => {
            app.prev_pane();
        }

        // Navigation: right pane
        KeyCode::Char('l') | KeyCode::Right => {
            app.next_pane();
        }

        // Tab: cycle panes
        KeyCode::Tab => {
            app.next_pane();
        }

        // Shift+Tab: reverse cycle panes
        KeyCode::BackTab => {
            app.prev_pane();
        }

        // Enter: select/activate
        KeyCode::Enter => {
            if let Err(e) = app.handle_enter(store) {
                app.set_error(format!("Action failed: {}", e));
            }
        }

        // Space: toggle accordion (when in filters pane on TagsHeader)
        KeyCode::Char(' ') => {
            if app.active_pane == app::ActivePane::Filters {
                if let Some(app::Filter::TagsHeader) = app.current_filter() {
                    app.toggle_tags_accordion();
                }
            }
        }

        // Command shortcuts
        KeyCode::Char('a') => {
            app.enter_command_mode(CommandType::Add);
        }
        KeyCode::Char('t') => {
            app.enter_command_mode(CommandType::Tag);
        }
        KeyCode::Char('n') => {
            app.enter_command_mode(CommandType::Note);
        }
        KeyCode::Char('e') => {
            app.enter_command_mode(CommandType::Edit);
        }
        KeyCode::Char('d') => {
            if let Err(e) = app.delete_current_link(store) {
                app.set_error(format!("Failed to delete link: {}", e));
            } else {
                return Ok(Some(true)); // Needs push
            }
        }
        KeyCode::Char('u') => {
            if let Err(e) = app.undo_delete(store) {
                app.set_error(format!("Failed to undo delete: {}", e));
            } else {
                return Ok(Some(true)); // Needs push
            }
        }

        // Filter mode
        KeyCode::Char('/') => {
            app.enter_filter_mode();
        }

        // Command mode
        KeyCode::Char(':') => {
            app.enter_command_mode(CommandType::Generic);
        }

        // Help
        KeyCode::Char('?') => {
            app.toggle_help();
        }

        // Manual sync
        KeyCode::Char('s') if modifiers.contains(KeyModifiers::CONTROL) => {
            return Ok(Some(true)); // Trigger push
        }

        // Vim navigation: G (go to last)
        KeyCode::Char('G') => {
            app.pending_g = None; // Clear any pending g
            app.move_to_last();
        }

        // Vim navigation: g (start of gg sequence)
        KeyCode::Char('g') => {
            if app.pending_g.is_some() {
                // Second 'g' - complete the gg sequence
                app.pending_g = None;
                app.move_to_first();
            } else {
                // First 'g' - start the sequence
                app.pending_g = Some(std::time::Instant::now());
            }
        }

        _ => {
            // Any other key clears pending 'g'
            app.pending_g = None;
        }
    }

    Ok(Some(false))
}

/// Handle key events in command mode
/// Returns Some(true) if local changes need to be pushed
async fn handle_command_mode<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    store: &mut Store,
    code: KeyCode,
    modifiers: KeyModifiers,
) -> Result<Option<bool>> {
    match code {
        // Cancel command
        KeyCode::Esc => {
            app.exit_input_mode();
        }
        KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => {
            app.exit_input_mode();
        }

        // Execute command
        KeyCode::Enter => {
            let result = match app.execute_command(store) {
                Ok(r) => r,
                Err(e) => {
                    app.set_error(format!("Command failed: {}", e));
                    app.exit_input_mode();
                    return Ok(Some(false));
                }
            };
            app.exit_input_mode();

            match result {
                CommandResult::Done => {}
                CommandResult::NeedMetadata(url) => {
                    // Check for duplicate URL first (before slow metadata fetch)
                    if let Ok(Some(existing)) = store.get_link_by_url(&url) {
                        app.set_status(format!("Link already exists: '{}'", existing.title));
                        return Ok(Some(false));
                    }

                    // Fetch metadata asynchronously
                    app.is_loading = true;
                    terminal.draw(|frame| ui::draw(frame, app))?;

                    let metadata = metadata::fetch_metadata(&url).await;
                    match app.add_link(store, &url, Some(metadata)) {
                        Ok(_) => {
                            app.is_loading = false;
                            return Ok(Some(true)); // Needs push
                        }
                        Err(e) => {
                            app.is_loading = false;
                            app.set_status(format!("Error: {}", e));
                            return Ok(Some(false));
                        }
                    }
                }
                CommandResult::NeedEditor(task) => {
                    // Exit TUI temporarily for editor
                    disable_raw_mode()?;
                    stdout().execute(LeaveAlternateScreen)?;
                    stdout().execute(cursor::Show)?;

                    let mut needs_push = false;

                    match task {
                        EditorTask::Note => {
                            let content =
                                match editor::edit_text("# Note\n\nEnter your note here...") {
                                    Ok(c) => c,
                                    Err(e) => {
                                        // Re-enter TUI before showing error
                                        enable_raw_mode()?;
                                        stdout().execute(EnterAlternateScreen)?;
                                        terminal.clear()?;
                                        app.set_error(format!("Editor failed: {}", e));
                                        return Ok(Some(false));
                                    }
                                };
                            let body: String = content
                                .lines()
                                .filter(|line| {
                                    let trimmed = line.trim();
                                    !trimmed.starts_with('#')
                                        && trimmed != "Enter your note here..."
                                })
                                .collect::<Vec<_>>()
                                .join("\n")
                                .trim()
                                .to_string();

                            enable_raw_mode()?;
                            stdout().execute(EnterAlternateScreen)?;
                            terminal.clear()?;

                            if !body.is_empty() {
                                if let Err(e) = app.add_note_to_current(store, &body) {
                                    app.set_error(format!("Failed to add note: {}", e));
                                } else {
                                    needs_push = true;
                                }
                            } else {
                                app.set_status("Note cancelled (empty)".to_string());
                            }
                        }
                        EditorTask::EditLink => {
                            if let Some(link) = app.current_link() {
                                let template = format!(
                                    "# Edit Link\n\
                                     # Lines starting with # are ignored\n\n\
                                     title: {}\n\
                                     url: {}\n\
                                     description: {}\n\
                                     tags: {}\n",
                                    link.title,
                                    link.url,
                                    link.description.as_deref().unwrap_or(""),
                                    link.tags.join(", ")
                                );

                                let content = match editor::edit_text(&template) {
                                    Ok(c) => c,
                                    Err(e) => {
                                        // Re-enter TUI before showing error
                                        enable_raw_mode()?;
                                        stdout().execute(EnterAlternateScreen)?;
                                        terminal.clear()?;
                                        app.set_error(format!("Editor failed: {}", e));
                                        return Ok(Some(false));
                                    }
                                };

                                enable_raw_mode()?;
                                stdout().execute(EnterAlternateScreen)?;
                                terminal.clear()?;

                                if let Some(updated) = parse_link_edit(&content, link) {
                                    if let Err(e) = store.update_link(&updated) {
                                        app.set_error(format!("Failed to update link: {}", e));
                                    } else if let Err(e) = app.refresh(store) {
                                        app.set_error(format!("Failed to refresh: {}", e));
                                    } else {
                                        app.set_status("Link updated".to_string());
                                        needs_push = true;
                                    }
                                } else {
                                    app.set_status("Edit cancelled".to_string());
                                }
                            } else {
                                enable_raw_mode()?;
                                stdout().execute(EnterAlternateScreen)?;
                                terminal.clear()?;
                                app.set_status("No link selected".to_string());
                            }
                        }
                    }

                    return Ok(Some(needs_push));
                }
            }
        }

        // Text input
        KeyCode::Char(c) => {
            app.insert_char(c);
        }
        KeyCode::Backspace => {
            app.delete_char();
        }
        KeyCode::Left => {
            app.cursor_left();
        }
        KeyCode::Right => {
            app.cursor_right();
        }

        _ => {}
    }

    Ok(Some(false))
}

/// Handle key events in filter mode
fn handle_filter_mode(app: &mut App, store: &Store, code: KeyCode) -> Result<()> {
    match code {
        // Cancel filter
        KeyCode::Esc => {
            app.exit_input_mode();
            if let Err(e) = app.clear_filter(store) {
                app.set_error(format!("Failed to clear filter: {}", e));
            }
        }

        // Confirm filter (stay in filtered view)
        KeyCode::Enter => {
            app.exit_input_mode();
        }

        // Text input
        KeyCode::Char(c) => {
            app.insert_char(c);
        }
        KeyCode::Backspace => {
            app.delete_char();
        }
        KeyCode::Left => {
            app.cursor_left();
        }
        KeyCode::Right => {
            app.cursor_right();
        }

        _ => {}
    }

    Ok(())
}

/// Parse edited link content from editor
fn parse_link_edit(content: &str, original: &rott_core::Link) -> Option<rott_core::Link> {
    let mut link = original.clone();
    let mut changed = false;

    for line in content.lines() {
        let line = line.trim();

        if line.starts_with('#') || line.is_empty() {
            continue;
        }

        if let Some(value) = line.strip_prefix("title:") {
            let value = value.trim();
            if value != link.title {
                link.set_title(value);
                changed = true;
            }
        } else if let Some(value) = line.strip_prefix("description:") {
            let value = value.trim();
            let new_desc = if value.is_empty() {
                None
            } else {
                Some(value.to_string())
            };
            if new_desc != link.description {
                link.set_description(new_desc);
                changed = true;
            }
        } else if let Some(value) = line.strip_prefix("tags:") {
            let tags: Vec<String> = value
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            if tags != link.tags {
                link.set_tags(tags);
                changed = true;
            }
        }
    }

    if changed {
        Some(link)
    } else {
        None
    }
}

/// Initialize logging for TUI mode
///
/// Only initializes if ROTT_LOG environment variable is set.
/// Logs to file (config.log_file or default {data_dir}/debug.log).
fn init_tui_logging(config: &Config) {
    // Only log if ROTT_LOG is set
    let Ok(log_level) = std::env::var("ROTT_LOG") else {
        return;
    };

    // Determine log file path
    let log_path = config
        .log_file
        .clone()
        .unwrap_or_else(|| config.data_dir.join("debug.log"));

    // Create log file
    let log_file = match File::create(&log_path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Warning: Could not create log file {:?}: {}", log_path, e);
            return;
        }
    };

    let env_filter = EnvFilter::new(format!("rott_core={},rott_cli={}", log_level, log_level));

    // Initialize file-based logging (ignore error if already initialized)
    let _ = tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_target(false)
        .with_ansi(false)
        .with_writer(log_file)
        .try_init();

    info!("TUI logging initialized to {:?}", log_path);
}
