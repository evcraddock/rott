//! UI rendering

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

use super::app::{ActivePane, App, Filter, InputMode, SyncIndicator};

fn display_width(value: &str) -> usize {
    Span::raw(value).width()
}

fn truncate_to_width(value: &str, max_width: usize) -> String {
    if display_width(value) <= max_width {
        return value.to_string();
    }

    const ELLIPSIS: &str = "…";
    let ellipsis_width = display_width(ELLIPSIS);
    if max_width < ellipsis_width {
        return String::new();
    }

    let content_width = max_width - ellipsis_width;
    let mut end = 0;
    for (index, character) in value.char_indices() {
        let candidate_end = index + character.len_utf8();
        if display_width(&value[..candidate_end]) > content_width {
            break;
        }
        end = candidate_end;
    }

    format!("{}{}", &value[..end], ELLIPSIS)
}

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

    // Draw sync indicator in top-right corner
    draw_sync_indicator(frame, app);

    // Draw status bar or command input
    match app.input_mode {
        InputMode::Normal => draw_status_bar(frame, app, outer_chunks[1]),
        InputMode::Command => draw_command_input(frame, app, outer_chunks[1]),
        InputMode::Filter => draw_filter_input(frame, app, outer_chunks[1]),
    }

    // Draw help overlay if visible
    if app.show_help {
        draw_help_overlay(frame);
    }

    // Draw device panel if visible
    if app.show_device_panel {
        draw_device_panel(frame, app);
    }

    // Draw error modal if there's an error (on top of everything)
    if let Some(ref error) = app.error_message {
        draw_error_modal(frame, error);
    }
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
            let max_width = area.width.saturating_sub(4) as usize;
            let title = truncate_to_width(&link.title, max_width);

            let url_max_width = max_width.saturating_sub(2);
            let url = truncate_to_width(&link.url, url_max_width);

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
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled(
                "Description: ",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw(link.description.as_deref().unwrap_or("-")),
        ]));

        // Author
        lines.push(Line::from(""));
        let author_str = if link.author.is_empty() {
            "-".to_string()
        } else {
            link.author.join(", ")
        };
        lines.push(Line::from(vec![
            Span::styled("Author: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(author_str),
        ]));

        // Tags
        lines.push(Line::from(""));
        let tags_str = if link.tags.is_empty() {
            "-".to_string()
        } else {
            link.tags.join(", ")
        };
        lines.push(Line::from(vec![
            Span::styled("Tags: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(tags_str),
        ]));

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

        // Notes section with separator
        lines.push(Line::from(""));
        if link.notes.is_empty() {
            lines.push(Line::from(vec![Span::styled(
                "── No notes ──",
                Style::default().add_modifier(Modifier::DIM),
            )]));
        } else {
            // Create separator line that fits width
            let note_header = format!("── Notes ({}) ", link.notes.len());
            let remaining = area.width.saturating_sub(note_header.len() as u16 + 2) as usize;
            let separator = format!("{}{}", note_header, "─".repeat(remaining));
            lines.push(Line::from(vec![Span::styled(
                separator,
                Style::default().add_modifier(Modifier::DIM),
            )]));

            for note in &link.notes {
                lines.push(Line::from(""));
                let timestamp = note.created_at.format("%Y-%m-%d").to_string();
                if let Some(title) = &note.title {
                    lines.push(Line::from(vec![
                        Span::styled(
                            format!("[{}] ", timestamp),
                            Style::default().add_modifier(Modifier::DIM),
                        ),
                        Span::styled(title, Style::default().add_modifier(Modifier::BOLD)),
                    ]));
                    // Show body indented below title
                    for body_line in note.body.lines() {
                        lines.push(Line::from(format!("  {}", body_line)));
                    }
                } else {
                    lines.push(Line::from(vec![Span::styled(
                        format!("[{}]", timestamp),
                        Style::default().add_modifier(Modifier::DIM),
                    )]));
                    // Show body indented
                    for body_line in note.body.lines() {
                        lines.push(Line::from(format!("  {}", body_line)));
                    }
                }
            }
        }

        lines
    } else {
        vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                "Select a link to view details",
                Style::default().add_modifier(Modifier::DIM),
            )]),
        ]
    };

    let paragraph = Paragraph::new(content)
        .block(block)
        .wrap(Wrap { trim: true })
        .scroll((app.detail_scroll, 0));

    frame.render_widget(paragraph, area);
}

/// Draw the status bar at the bottom
fn draw_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let content = if app.is_loading {
        "Adding link...".to_string()
    } else if let Some(msg) = &app.status_message {
        msg.clone()
    } else {
        "a:add  t:tag  n:note  e:edit  d:del  u:undo  /:filter  ?:help  q:quit".to_string()
    };

    let paragraph = Paragraph::new(content).style(Style::default().add_modifier(Modifier::DIM));

    frame.render_widget(paragraph, area);
}

/// Draw command input at the bottom
fn draw_command_input(frame: &mut Frame, app: &App, area: Rect) {
    // Build the input line with cursor
    let prefix = ":";
    let input = &app.command_input;

    let line = Line::from(vec![
        Span::styled(prefix, Style::default().fg(Color::Yellow)),
        Span::raw(input.as_str()),
    ]);

    let paragraph = Paragraph::new(line);
    frame.render_widget(paragraph, area);

    // Position cursor
    let cursor_x = area.x + prefix.len() as u16 + app.command_cursor as u16;
    frame.set_cursor_position((cursor_x, area.y));
}

/// Draw filter input at the bottom
fn draw_filter_input(frame: &mut Frame, app: &App, area: Rect) {
    let prefix = "/";
    let input = &app.command_input;

    let line = Line::from(vec![
        Span::styled(prefix, Style::default().fg(Color::Cyan)),
        Span::raw(input.as_str()),
        Span::styled(
            format!("  ({} matches)", app.links.len()),
            Style::default().add_modifier(Modifier::DIM),
        ),
    ]);

    let paragraph = Paragraph::new(line);
    frame.render_widget(paragraph, area);

    // Position cursor
    let cursor_x = area.x + prefix.len() as u16 + app.command_cursor as u16;
    frame.set_cursor_position((cursor_x, area.y));
}

/// Draw sync indicator in top-right corner
fn draw_sync_indicator(frame: &mut Frame, app: &App) {
    let area = frame.area();
    if area.width < 5 {
        return;
    }

    let (icon, style) = match app.sync_status {
        SyncIndicator::Synced => ("✓", Style::default().fg(Color::Green)),
        SyncIndicator::Syncing => ("↻", Style::default().fg(Color::Yellow)),
        SyncIndicator::Offline => ("⚡", Style::default().fg(Color::DarkGray)),
        SyncIndicator::Disabled => ("○", Style::default().add_modifier(Modifier::DIM)),
        SyncIndicator::Error => ("✗", Style::default().fg(Color::Red)),
    };

    let indicator = Paragraph::new(Span::styled(icon, style));
    let indicator_area = Rect::new(area.width - 2, 0, 1, 1);
    frame.render_widget(indicator, indicator_area);
}

/// Draw help overlay
fn draw_help_overlay(frame: &mut Frame) {
    let area = frame.area();

    // Calculate centered popup area
    let popup_width = 50.min(area.width.saturating_sub(4));
    let popup_height = 20.min(area.height.saturating_sub(4));
    let popup_x = (area.width.saturating_sub(popup_width)) / 2;
    let popup_y = (area.height.saturating_sub(popup_height)) / 2;
    let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

    // Clear the popup area
    frame.render_widget(ratatui::widgets::Clear, popup_area);

    let help_text = vec![
        Line::from(vec![Span::styled(
            "Keyboard Shortcuts",
            Style::default().add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from("Navigation:"),
        Line::from("  j/k, ↑/↓    Move up/down"),
        Line::from("  gg          Jump to first item"),
        Line::from("  G           Jump to last item"),
        Line::from("  h/l, ←/→    Switch panes"),
        Line::from("  Tab         Cycle panes"),
        Line::from("  Enter       Open link / Apply filter"),
        Line::from(""),
        Line::from("Commands:"),
        Line::from("  a           Add link"),
        Line::from("  t           Edit tags"),
        Line::from("  n           Add note"),
        Line::from("  e           Edit link"),
        Line::from("  d           Delete link"),
        Line::from("  u           Undo delete"),
        Line::from(""),
        Line::from("  /           Filter view"),
        Line::from("  :           Command mode"),
        Line::from("  Ctrl+D      Device settings"),
        Line::from("  q           Quit"),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Press any key to close",
            Style::default().add_modifier(Modifier::DIM),
        )]),
    ];

    let block = Block::default()
        .title(" Help ")
        .borders(Borders::ALL)
        .border_style(Style::default().add_modifier(Modifier::BOLD));

    let paragraph = Paragraph::new(help_text).block(block);
    frame.render_widget(paragraph, popup_area);
}

/// Draw device settings panel
fn draw_device_panel(frame: &mut Frame, app: &App) {
    let area = frame.area();

    // Calculate centered popup area
    let popup_width = 70.min(area.width.saturating_sub(4));
    let popup_height = 18.min(area.height.saturating_sub(4));
    let popup_x = (area.width.saturating_sub(popup_width)) / 2;
    let popup_y = (area.height.saturating_sub(popup_height)) / 2;
    let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

    // Clear the popup area
    frame.render_widget(ratatui::widgets::Clear, popup_area);

    // Sync status display
    let sync_status_str = match app.sync_status {
        SyncIndicator::Synced => ("✓ Connected", Color::Green),
        SyncIndicator::Syncing => ("↻ Syncing", Color::Yellow),
        SyncIndicator::Offline => ("⚡ Offline", Color::Gray),
        SyncIndicator::Disabled => ("○ Disabled", Color::Gray),
        SyncIndicator::Error => ("✗ Error", Color::Red),
    };

    let device_text = vec![
        Line::from(vec![Span::styled(
            "Device Information",
            Style::default().add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Root Document ID: ",
            Style::default().add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(
                &app.device_info.root_id,
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "Sync Server: ",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw(
                app.device_info
                    .sync_url
                    .as_deref()
                    .unwrap_or("Not configured"),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "Sync Status: ",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::styled(sync_status_str.0, Style::default().fg(sync_status_str.1)),
        ]),
        Line::from(""),
        Line::from(""),
        Line::from(vec![
            Span::styled("[y] ", Style::default().fg(Color::Yellow)),
            Span::raw("Copy ID to clipboard"),
        ]),
        Line::from(vec![
            Span::styled("[Esc] ", Style::default().fg(Color::Yellow)),
            Span::raw("Close"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Use this ID to set up ROTT on other devices",
            Style::default().add_modifier(Modifier::DIM),
        )]),
    ];

    let block = Block::default()
        .title(" Device Settings (Ctrl+D) ")
        .borders(Borders::ALL)
        .border_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        );

    let paragraph = Paragraph::new(device_text).block(block);
    frame.render_widget(paragraph, popup_area);
}

/// Draw error modal
fn draw_error_modal(frame: &mut Frame, error: &str) {
    let area = frame.area();

    // Calculate centered popup area (smaller than help)
    let popup_width = 60.min(area.width.saturating_sub(4));
    let popup_height = 10.min(area.height.saturating_sub(4));
    let popup_x = (area.width.saturating_sub(popup_width)) / 2;
    let popup_y = (area.height.saturating_sub(popup_height)) / 2;
    let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

    // Clear the popup area
    frame.render_widget(ratatui::widgets::Clear, popup_area);

    // Build error text with word wrapping
    let mut lines = vec![
        Line::from(""),
        Line::from(vec![Span::styled(error, Style::default().fg(Color::White))]),
    ];

    // Add spacing and dismiss instruction
    lines.push(Line::from(""));
    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::styled(
        "Press any key to dismiss",
        Style::default().fg(Color::Gray).add_modifier(Modifier::DIM),
    )]));

    let block = Block::default()
        .title(" Error ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD));

    let paragraph = Paragraph::new(lines).block(block).wrap(Wrap { trim: true });
    frame.render_widget(paragraph, popup_area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{backend::TestBackend, Terminal};
    use rott_core::Link;

    const REPORTED_TITLE: &str = "GitHub - geastack/examples: Example Gea applications and tools — the app gallery used by the simulator, the embedded targets, GeaOS and the Apple targets.";

    #[test]
    fn truncates_reported_title_at_utf8_boundaries() {
        for width in 60..=68 {
            let truncated = truncate_to_width(REPORTED_TITLE, width);

            let prefix = truncated
                .strip_suffix('…')
                .expect("truncated title should end with an ellipsis");
            assert!(REPORTED_TITLE.starts_with(prefix));
            assert!(REPORTED_TITLE.is_char_boundary(prefix.len()));
            assert!(display_width(&truncated) <= width);
        }
    }

    #[test]
    fn truncates_other_multibyte_characters_by_display_width() {
        assert_eq!(truncate_to_width("éclair", 2), "é…");
        assert_eq!(truncate_to_width("a界b", 3), "a…");
        assert_eq!(truncate_to_width("🙂rust", 4), "🙂r…");
        assert_eq!(truncate_to_width("a界b", 4), "a界b");
    }

    #[test]
    fn handles_narrow_widths() {
        assert_eq!(truncate_to_width("title", 0), "");
        assert_eq!(truncate_to_width("title", 1), "…");
        assert_eq!(truncate_to_width("", 0), "");
    }

    #[test]
    fn renders_multibyte_titles_while_resizing() {
        let app = app_with_title(REPORTED_TITLE);
        let backend = TestBackend::new(200, 10);
        let mut terminal = Terminal::new(backend).expect("test terminal should initialize");

        for width in [200, 140, 80, 20, 5] {
            terminal.backend_mut().resize(width, 10);
            terminal.autoresize().expect("terminal should resize");
            terminal
                .draw(|frame| draw(frame, &app))
                .expect("multibyte title should render");
        }
    }

    fn app_with_title(title: &str) -> App {
        let mut link = Link::new("https://example.com/路径/🙂");
        link.set_title(title);
        let links = vec![link];

        App {
            should_quit: false,
            input_mode: InputMode::Normal,
            command_type: None,
            command_input: String::new(),
            command_cursor: 0,
            active_pane: ActivePane::Items,
            filters: vec![Filter::Favorites],
            filter_index: 0,
            tags_expanded: false,
            all_tags: Vec::new(),
            all_links: links.clone(),
            links,
            link_index: 0,
            status_message: None,
            deleted_link: None,
            filter_text: String::new(),
            is_loading: false,
            detail_scroll: 0,
            status_message_time: None,
            show_help: false,
            sync_status: SyncIndicator::Disabled,
            pending_g: None,
            error_message: None,
            show_device_panel: false,
            device_info: super::super::app::DeviceInfo {
                root_id: String::new(),
                sync_url: None,
            },
        }
    }
}
