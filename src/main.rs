mod app;

use app::{ui, ActivePane, App};
use crossterm::{
    event::{self, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    prelude::*,
    widgets::{Block, Paragraph},
};

use std::io::{self, stdout};

fn main() -> io::Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    // Create app state
    let mut app = App::new();
    app.topics.state.select(Some(0));
    app.pages.state.select(Some(0));

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
                            }
                        }
                        KeyCode::BackTab | KeyCode::Char('h') => {
                            if app.active_pane == ActivePane::Pages {
                                app.active_pane = ActivePane::Topics;
                            }
                        }
                        KeyCode::Down | KeyCode::Char('j') => match app.active_pane {
                            ActivePane::Topics => app.topics.next(),
                            ActivePane::Pages => app.pages.next(),
                        },
                        KeyCode::Up | KeyCode::Char('k') => match app.active_pane {
                            ActivePane::Topics => app.topics.previous(),
                            ActivePane::Pages => app.pages.previous(),
                        },
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
