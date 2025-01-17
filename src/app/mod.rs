mod handlers;
mod models;

// pub use handlers::*;
pub use models::*;

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem},
};

pub fn ui(frame: &mut Frame, app: &mut App) {
    // Create layout with two equal horizontal splits
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(25), Constraint::Percentage(75)])
        .split(frame.area());

    // Topics list
    let topics: Vec<ListItem> = app
        .topics
        .items
        .iter()
        .map(|topic| ListItem::new(topic.as_str()))
        .collect();

    let topics_block = Block::default()
        .title("Topics")
        .borders(Borders::ALL)
        .border_style(if app.active_pane == ActivePane::Topics {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        });

    let topics_list = List::new(topics)
        .block(topics_block)
        .highlight_style(Style::default().bg(Color::DarkGray));

    frame.render_stateful_widget(topics_list, chunks[0], &mut app.topics.state);

    // Pages list
    let pages: Vec<ListItem> = app
        .pages
        .items
        .iter()
        .map(|page| ListItem::new(page.title.as_str()))
        .collect();

    let pages_block = Block::default()
        .title("Pages")
        .borders(Borders::ALL)
        .border_style(if app.active_pane == ActivePane::Pages {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        });

    let pages_list = List::new(pages)
        .block(pages_block)
        .highlight_style(Style::default().bg(Color::DarkGray));

    frame.render_stateful_widget(pages_list, chunks[1], &mut app.pages.state);
}
