//! ROTT TUI
//!
//! Terminal user interface for ROTT - links and notes management.

use anyhow::Result;
use crossterm::{
    event::{self, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};
use std::io::stdout;

fn main() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    // Run app
    let result = run_app(&mut terminal);

    // Restore terminal
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;

    result
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>) -> Result<()> {
    loop {
        terminal.draw(|frame| {
            let area = frame.area();

            let block = Block::default()
                .title(" ROTT - Links and Notes ")
                .borders(Borders::ALL);

            let text = Paragraph::new("Press 'q' to quit\n\nTUI implementation coming in Phase 4")
                .block(block)
                .alignment(Alignment::Center);

            frame.render_widget(text, area);
        })?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let event::Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                    break;
                }
            }
        }
    }

    Ok(())
}
