mod models;
pub use models::*;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
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
        .map(|page| {
            ListItem::new(format!(
                "{}\n{}\n\n",
                page.title,
                page.source.as_deref().unwrap_or("")
            ))
        })
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
        .style(Style::default())
        .highlight_style(Style::default().bg(Color::DarkGray));

    frame.render_stateful_widget(pages_list, chunks[1], &mut app.pages.state);

    // Render tag editing popup if in editing mode
    if app.input_mode == InputMode::EditingTags {
        let area = frame.area();
        let popup_area = centered_rect(60, 20, area);

        // Clear the area
        frame.render_widget(Clear, popup_area);

        // Create the input box
        let input = Paragraph::new(app.tag_input.as_str())
            .style(Style::default().fg(Color::Yellow))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Edit Tags (comma-separated)")
                    .border_style(Style::default().fg(Color::Yellow)),
            );

        frame.render_widget(input, popup_area);

        // Show cursor
        frame.set_cursor_position(Position::new(
            popup_area.x + app.tag_input.len() as u16 + 1,
            popup_area.y + 1,
        ));
    }
}

/// Helper function to create a centered rect
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
