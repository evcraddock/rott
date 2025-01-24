#![allow(dead_code)]
mod app;
mod links;

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

fn main_debug() {
    App::new(None);
}

fn main() -> io::Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    // Create app state
    let mut app = App::new(None);
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
                                app = App::new(None);
                                app.topics.state.select(Some(0));
                            }
                        }
                        KeyCode::Enter => {
                            if app.active_pane == ActivePane::Pages {
                                if let Some(index) = app.pages.state.selected() {
                                    if let Some(url) = app.pages.items[index].source.clone() {
                                        let _ = open::that(url); // Ignore potential error
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
