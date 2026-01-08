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

mod app;
mod ui;

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::prelude::*;
use rott_core::Store;
use std::io::stdout;

use app::App;

fn main() -> Result<()> {
    // Open the store
    let store = Store::open()?;

    // Setup terminal
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    // Create app
    let mut app = App::new(&store)?;

    // Apply initial filter (Recent)
    app.apply_filter(&store)?;

    // Run app
    let result = run_app(&mut terminal, &mut app, &store);

    // Restore terminal
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;

    result
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App, store: &Store) -> Result<()> {
    loop {
        // Draw UI
        terminal.draw(|frame| ui::draw(frame, app))?;

        // Handle events
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                // Only handle key press events (not release)
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                // Clear status message on any key press
                app.status_message = None;

                match key.code {
                    // Quit
                    KeyCode::Char('q') => {
                        app.should_quit = true;
                    }
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
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

                    // Help
                    KeyCode::Char('?') => {
                        app.status_message = Some(
                            "j/k:↑↓  h/l:←→  Tab:cycle  Enter:open  Space:expand  q:quit"
                                .to_string(),
                        );
                    }

                    _ => {}
                }
            }
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}
