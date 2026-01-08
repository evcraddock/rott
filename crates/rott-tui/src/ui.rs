//! UI rendering

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

use crate::app::{ActivePane, App, Filter};

/// Main UI rendering function
pub fn draw(frame: &mut Frame, app: &App) {
    // Create vertical layout for status bar at the bottom
    let outer_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(1)])
        .split(frame.area());

    // Split the main area into three panes
    let pane_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Percentage(35),
            Constraint::Percentage(45),
        ])
        .split(outer_chunks[0]);

    // Render each pane
    draw_filters_pane(frame, app, pane_chunks[0]);
    draw_items_pane(frame, app, pane_chunks[1]);
    draw_detail_pane(frame, app, pane_chunks[2]);
    draw_status_bar(frame, app, outer_chunks[1]);
}

/// Draw the filters pane (left)
fn draw_filters_pane(frame: &mut Frame, app: &App, area: Rect) {
    let is_active = app.active_pane == ActivePane::Filters;

    let items: Vec<ListItem> = app
        .filters
        .iter()
        .map(|filter| {
            let name = match filter {
                Filter::Favorites => "★ Favorites".to_string(),
                Filter::Recent => "⏱ Recent".to_string(),
                Filter::Untagged => "○ Untagged".to_string(),
                Filter::TagsHeader => {
                    if app.tags_expanded {
                        "▼ By Tag...".to_string()
                    } else {
                        "▶ By Tag...".to_string()
                    }
                }
                Filter::ByTag(tag) => format!("    #{}", tag),
            };

            ListItem::new(name)
        })
        .collect();

    let border_style = if is_active {
        Style::default().add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };

    let block = Block::default()
        .title(" Filters ")
        .borders(Borders::ALL)
        .border_style(border_style);

    let highlight_style = if is_active {
        Style::default()
            .add_modifier(Modifier::BOLD)
            .add_modifier(Modifier::REVERSED)
    } else {
        Style::default().add_modifier(Modifier::REVERSED)
    };

    let list = List::new(items)
        .block(block)
        .highlight_style(highlight_style);

    let mut state = ListState::default();
    state.select(Some(app.filter_index));

    frame.render_stateful_widget(list, area, &mut state);
}

/// Draw the items pane (middle)
fn draw_items_pane(frame: &mut Frame, app: &App, area: Rect) {
    let is_active = app.active_pane == ActivePane::Items;

    let items: Vec<ListItem> = app
        .links
        .iter()
        .map(|link| {
            // Truncate title if too long
            let max_len = area.width.saturating_sub(4) as usize;
            let title = if link.title.len() > max_len {
                format!("{}…", &link.title[..max_len.saturating_sub(1)])
            } else {
                link.title.clone()
            };

            // Truncate URL
            let url_max = max_len.saturating_sub(2);
            let url = if link.url.len() > url_max {
                format!("{}…", &link.url[..url_max.saturating_sub(1)])
            } else {
                link.url.clone()
            };

            let content = Line::from(vec![Span::styled(title, Style::default())]);

            let url_line = Line::from(vec![Span::styled(
                url,
                Style::default().add_modifier(Modifier::DIM),
            )]);

            ListItem::new(vec![content, url_line])
        })
        .collect();

    let border_style = if is_active {
        Style::default().add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };

    let title = format!(" Items ({}) ", app.links.len());
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(border_style);

    let highlight_style = if is_active {
        Style::default()
            .add_modifier(Modifier::BOLD)
            .add_modifier(Modifier::REVERSED)
    } else {
        Style::default().add_modifier(Modifier::REVERSED)
    };

    let list = List::new(items)
        .block(block)
        .highlight_style(highlight_style);

    let mut state = ListState::default();
    if !app.links.is_empty() {
        state.select(Some(app.link_index));
    }

    frame.render_stateful_widget(list, area, &mut state);
}

/// Draw the detail pane (right)
fn draw_detail_pane(frame: &mut Frame, app: &App, area: Rect) {
    let is_active = app.active_pane == ActivePane::Detail;

    let border_style = if is_active {
        Style::default().add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };

    let block = Block::default()
        .title(" Detail ")
        .borders(Borders::ALL)
        .border_style(border_style);

    let content = if let Some(link) = app.current_link() {
        let mut lines = vec![
            Line::from(vec![
                Span::styled("Title: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(&link.title),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("URL: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(&link.url),
            ]),
        ];

        // Description
        if let Some(desc) = &link.description {
            lines.push(Line::from(""));
            lines.push(Line::from(vec![Span::styled(
                "Description: ",
                Style::default().add_modifier(Modifier::BOLD),
            )]));
            lines.push(Line::from(desc.as_str()));
        }

        // Tags
        if !link.tags.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled("Tags: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(link.tags.join(", ")),
            ]));
        }

        // Authors
        if !link.author.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled("Author: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(link.author.join(", ")),
            ]));
        }

        // Dates
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("Created: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(link.created_at.format("%Y-%m-%d %H:%M").to_string()),
        ]));
        lines.push(Line::from(vec![
            Span::styled("Updated: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(link.updated_at.format("%Y-%m-%d %H:%M").to_string()),
        ]));

        // Notes
        if !link.notes.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(vec![Span::styled(
                format!("Notes ({}):", link.notes.len()),
                Style::default().add_modifier(Modifier::BOLD),
            )]));

            for note in &link.notes {
                lines.push(Line::from(""));
                let timestamp = note.created_at.format("%Y-%m-%d %H:%M").to_string();
                if let Some(title) = &note.title {
                    lines.push(Line::from(vec![
                        Span::styled(
                            format!("  {} - ", timestamp),
                            Style::default().add_modifier(Modifier::DIM),
                        ),
                        Span::styled(title, Style::default().add_modifier(Modifier::ITALIC)),
                    ]));
                } else {
                    lines.push(Line::from(vec![Span::styled(
                        format!("  {}", timestamp),
                        Style::default().add_modifier(Modifier::DIM),
                    )]));
                }
                lines.push(Line::from(format!("  {}", note.body)));
            }
        }

        lines
    } else {
        vec![Line::from("No link selected")]
    };

    let paragraph = Paragraph::new(content)
        .block(block)
        .wrap(Wrap { trim: true });

    frame.render_widget(paragraph, area);
}

/// Draw the status bar at the bottom
fn draw_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let content = if let Some(msg) = &app.status_message {
        msg.clone()
    } else {
        "j/k:nav  h/l:pane  Enter:select/open  Tab:next pane  q:quit  ?:help".to_string()
    };

    let paragraph = Paragraph::new(content).style(Style::default().add_modifier(Modifier::DIM));

    frame.render_widget(paragraph, area);
}
