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
mod editor;
mod metadata;
mod sync;
mod ui;

use anyhow::Result;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::prelude::*;
use rott_core::Store;
use std::io::stdout;

use app::{App, CommandResult, CommandType, EditorTask, InputMode, SyncIndicator};
use sync::{SyncCommand, SyncEvent, SyncHandle};

#[tokio::main]
async fn main() -> Result<()> {
    // Open the store
    let mut store = Store::open()?;
    let config = store.config().clone();

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

        // Initial sync to get latest data
        app.sync_status = sync::do_sync(&mut store).await;
        app.refresh(&store)?;

        // Spawn periodic sync poller (checks for remote changes every 5 seconds)
        Some(sync::spawn_sync_poller(5))
    } else {
        None
    };

    // Apply initial filter (Recent)
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
    mut sync_handle: Option<SyncHandle>,
) -> Result<()> {
    // Track if we need to sync after this iteration
    let mut pending_sync = false;

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
                    std::future::pending::<Option<SyncEvent>>().await
                }
            } => {
                if let Some(event) = sync_event {
                    match event {
                        SyncEvent::Connected => {
                            if app.sync_status != SyncIndicator::Syncing {
                                app.sync_status = SyncIndicator::Synced;
                            }
                        }
                        SyncEvent::Disconnected => {
                            app.sync_status = SyncIndicator::Offline;
                        }
                        SyncEvent::RemoteChanges => {
                            // Poll triggered - sync to check for remote changes
                            app.sync_status = SyncIndicator::Syncing;
                            terminal.draw(|frame| ui::draw(frame, app))?;

                            app.sync_status = sync::do_sync(store).await;
                            app.refresh(store)?;
                        }
                        SyncEvent::Error(msg) => {
                            app.set_status(format!("Sync error: {}", msg));
                            app.sync_status = SyncIndicator::Error;
                        }
                    }
                }
            }

            // Poll for terminal events
            _ = tokio::time::sleep(std::time::Duration::from_millis(50)) => {
                // Do pending sync if needed
                if pending_sync && sync::is_sync_enabled(store.config()) {
                    pending_sync = false;
                    let prev_status = app.sync_status;
                    app.sync_status = SyncIndicator::Syncing;
                    terminal.draw(|frame| ui::draw(frame, app))?;

                    app.sync_status = sync::do_sync(store).await;
                    app.refresh(store)?; // Refresh UI after sync!

                    // If sync failed but we were connected, show offline
                    if app.sync_status == SyncIndicator::Offline && prev_status == SyncIndicator::Synced {
                        // Keep offline status
                    }
                }

                // Check for terminal events (non-blocking)
                if event::poll(std::time::Duration::from_millis(0))? {
                    if let Event::Key(key) = event::read()? {
                        // Only handle key press events (not release)
                        if key.kind != KeyEventKind::Press {
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
                                if let Some(needs_sync) = handle_normal_mode(app, store, key.code, key.modifiers).await? {
                                    if needs_sync {
                                        pending_sync = true;
                                    }
                                }
                            }
                            InputMode::Command => {
                                if let Some(needs_sync) = handle_command_mode(terminal, app, store, key.code, key.modifiers).await? {
                                    if needs_sync {
                                        pending_sync = true;
                                    }
                                }
                            }
                            InputMode::Filter => handle_filter_mode(app, store, key.code)?,
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
/// Returns Some(true) if a sync is needed, Some(false) if not, None for no action
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
        | KeyCode::BackTab => {
            app.status_message = None;
        }
        _ => {}
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
            app.handle_enter(store)?;
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
            app.delete_current_link(store)?;
            return Ok(Some(true)); // Needs sync
        }
        KeyCode::Char('u') => {
            app.undo_delete(store)?;
            return Ok(Some(true)); // Needs sync
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
            return Ok(Some(true)); // Trigger sync
        }

        _ => {}
    }

    Ok(Some(false))
}

/// Handle key events in command mode
/// Returns Some(true) if a sync is needed
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
            let result = app.execute_command(store)?;
            app.exit_input_mode();

            match result {
                CommandResult::Done => {}
                CommandResult::NeedMetadata(url) => {
                    // Fetch metadata asynchronously
                    app.is_loading = true;
                    terminal.draw(|frame| ui::draw(frame, app))?;

                    let metadata = metadata::fetch_metadata(&url).await;
                    app.add_link(store, &url, metadata)?;
                    app.is_loading = false;

                    return Ok(Some(true)); // Needs sync
                }
                CommandResult::NeedEditor(task) => {
                    // Exit TUI temporarily for editor
                    disable_raw_mode()?;
                    stdout().execute(LeaveAlternateScreen)?;
                    stdout().execute(cursor::Show)?;

                    let mut needs_sync = false;

                    match task {
                        EditorTask::Note => {
                            let content = editor::edit_text("# Note\n\nEnter your note here...")?;
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
                                app.add_note_to_current(store, &body)?;
                                needs_sync = true;
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

                                let content = editor::edit_text(&template)?;

                                enable_raw_mode()?;
                                stdout().execute(EnterAlternateScreen)?;
                                terminal.clear()?;

                                if let Some(updated) = parse_link_edit(&content, link) {
                                    store.update_link(&updated)?;
                                    app.set_status("Link updated".to_string());
                                    app.refresh(store)?;
                                    needs_sync = true;
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

                    return Ok(Some(needs_sync));
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
            app.clear_filter(store)?;
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
