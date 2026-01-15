//! Setup wizard for first-run device configuration
//!
//! Handles:
//! - Welcome screen with create/join options
//! - New identity creation with ID display
//! - Join existing identity with ID input
//! - Sync progress for join flow

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};
use rott_core::{Config, DocumentId, Identity, Store};

/// Setup wizard state
pub struct SetupWizard {
    /// Current screen
    screen: Screen,
    /// Input buffer for join flow
    input: String,
    /// Cursor position in input
    cursor: usize,
    /// Error message to display
    error: Option<String>,
    /// Generated root document ID (for new identity flow)
    generated_id: Option<DocumentId>,
    /// Whether user has acknowledged saving the ID
    id_acknowledged: bool,
    /// Config for sync operations
    config: Config,
}

/// Wizard screens
#[derive(Debug, Clone, PartialEq, Eq)]
enum Screen {
    /// Welcome screen with create/join choice
    Welcome,
    /// New identity - showing generated ID
    NewIdentity,
    /// Join existing - ID input
    JoinInput,
    /// Join existing - syncing
    JoinSyncing,
    /// Setup complete
    Complete,
}

/// Result of running the wizard
pub enum SetupResult {
    /// Setup completed successfully
    Complete,
    /// User quit the wizard
    Quit,
}

impl SetupWizard {
    /// Create a new setup wizard
    pub fn new(config: Config) -> Self {
        Self {
            screen: Screen::Welcome,
            input: String::new(),
            cursor: 0,
            error: None,
            generated_id: None,
            id_acknowledged: false,
            config,
        }
    }

    /// Run the setup wizard
    pub async fn run(
        &mut self,
        terminal: &mut ratatui::Terminal<impl ratatui::backend::Backend>,
    ) -> Result<SetupResult> {
        loop {
            // Draw current screen
            terminal.draw(|frame| self.draw(frame))?;

            // Handle input
            if event::poll(std::time::Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind != KeyEventKind::Press {
                        continue;
                    }

                    // Clear error on any keypress
                    if self.error.is_some() && self.screen != Screen::JoinInput {
                        self.error = None;
                        continue;
                    }

                    match self.handle_key(key.code, key.modifiers).await? {
                        Some(SetupResult::Complete) => return Ok(SetupResult::Complete),
                        Some(SetupResult::Quit) => return Ok(SetupResult::Quit),
                        None => {}
                    }
                }
            }

            // Check if syncing completed
            if self.screen == Screen::JoinSyncing {
                // Sync is handled in handle_key, this is just for UI updates
            }
        }
    }

    /// Handle key input, returns Some if wizard should exit
    async fn handle_key(
        &mut self,
        code: KeyCode,
        modifiers: KeyModifiers,
    ) -> Result<Option<SetupResult>> {
        // Global quit
        if code == KeyCode::Char('c') && modifiers.contains(KeyModifiers::CONTROL) {
            return Ok(Some(SetupResult::Quit));
        }

        match &self.screen {
            Screen::Welcome => self.handle_welcome(code),
            Screen::NewIdentity => self.handle_new_identity(code),
            Screen::JoinInput => self.handle_join_input(code, modifiers).await,
            Screen::JoinSyncing => Ok(None), // No input during sync
            Screen::Complete => Ok(Some(SetupResult::Complete)),
        }
    }

    fn handle_welcome(&mut self, code: KeyCode) -> Result<Option<SetupResult>> {
        match code {
            KeyCode::Char('1') | KeyCode::Char('n') | KeyCode::Char('N') => {
                // Create new identity
                let identity = Identity::with_config(self.config.clone());
                match identity.initialize_new() {
                    Ok(result) => {
                        self.generated_id = Some(result.root_id);
                        self.screen = Screen::NewIdentity;
                    }
                    Err(e) => {
                        self.error = Some(format!("Failed to create identity: {}", e));
                    }
                }
            }
            KeyCode::Char('2') | KeyCode::Char('j') | KeyCode::Char('J') => {
                self.screen = Screen::JoinInput;
            }
            KeyCode::Char('q') | KeyCode::Esc => {
                return Ok(Some(SetupResult::Quit));
            }
            _ => {}
        }
        Ok(None)
    }

    fn handle_new_identity(&mut self, code: KeyCode) -> Result<Option<SetupResult>> {
        match code {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                // Copy to clipboard
                if let Some(id) = &self.generated_id {
                    if copy_to_clipboard(&id.to_string()) {
                        self.error = Some("Copied to clipboard!".to_string());
                    } else {
                        self.error = Some("Clipboard not available - copy manually".to_string());
                    }
                }
            }
            KeyCode::Enter | KeyCode::Char('c') | KeyCode::Char('C') => {
                if !self.id_acknowledged {
                    self.id_acknowledged = true;
                } else {
                    self.screen = Screen::Complete;
                    return Ok(Some(SetupResult::Complete));
                }
            }
            KeyCode::Char('q') | KeyCode::Esc => {
                // Don't allow quit without acknowledging
                if !self.id_acknowledged {
                    self.error =
                        Some("Please save your ID first! Press Enter to confirm.".to_string());
                } else {
                    self.screen = Screen::Complete;
                    return Ok(Some(SetupResult::Complete));
                }
            }
            _ => {}
        }
        Ok(None)
    }

    async fn handle_join_input(
        &mut self,
        code: KeyCode,
        modifiers: KeyModifiers,
    ) -> Result<Option<SetupResult>> {
        match code {
            KeyCode::Esc => {
                self.screen = Screen::Welcome;
                self.input.clear();
                self.cursor = 0;
                self.error = None;
            }
            KeyCode::Enter => {
                // Validate and attempt join
                let input = self.input.trim();
                if input.is_empty() {
                    self.error = Some("Please enter a root document ID".to_string());
                    return Ok(None);
                }

                // Parse the document ID (supports both raw bs58check and automerge: URL format)
                let parse_result = if input.starts_with("automerge:") {
                    DocumentId::from_url(input)
                } else {
                    DocumentId::from_bs58check(input)
                };
                match parse_result {
                    Ok(doc_id) => {
                        // Initialize with the joined ID
                        let identity = Identity::with_config(self.config.clone());
                        if let Err(e) = identity.initialize_join(doc_id) {
                            self.error = Some(format!("Failed to join: {}", e));
                            return Ok(None);
                        }

                        // Check if sync is configured
                        if self.config.sync_enabled && self.config.sync_url.is_some() {
                            self.screen = Screen::JoinSyncing;
                            self.error = None;

                            // Attempt sync
                            match Store::initial_sync(&self.config).await {
                                Ok(()) => {
                                    self.screen = Screen::Complete;
                                    return Ok(Some(SetupResult::Complete));
                                }
                                Err(e) => {
                                    self.error = Some(format!(
                                        "Sync failed: {}. You can sync later with 'rott sync'.",
                                        e
                                    ));
                                    self.screen = Screen::Complete;
                                    return Ok(Some(SetupResult::Complete));
                                }
                            }
                        } else {
                            // No sync configured - just save the ID
                            self.error =
                                Some("ID saved. Configure sync to pull your data.".to_string());
                            self.screen = Screen::Complete;
                            return Ok(Some(SetupResult::Complete));
                        }
                    }
                    Err(e) => {
                        self.error = Some(format!("Invalid ID format: {}", e));
                    }
                }
            }
            KeyCode::Char('v') if modifiers.contains(KeyModifiers::CONTROL) => {
                // Paste from clipboard
                if let Some(text) = paste_from_clipboard() {
                    self.input = text.trim().to_string();
                    self.cursor = self.input.len();
                    self.error = None;
                }
            }
            KeyCode::Char(c) => {
                self.input.insert(self.cursor, c);
                self.cursor += 1;
                self.error = None;
            }
            KeyCode::Backspace => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                    self.input.remove(self.cursor);
                    self.error = None;
                }
            }
            KeyCode::Left => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                }
            }
            KeyCode::Right => {
                if self.cursor < self.input.len() {
                    self.cursor += 1;
                }
            }
            KeyCode::Home => {
                self.cursor = 0;
            }
            KeyCode::End => {
                self.cursor = self.input.len();
            }
            _ => {}
        }
        Ok(None)
    }

    /// Draw the current screen
    fn draw(&self, frame: &mut Frame) {
        let area = frame.area();

        // Clear the screen
        frame.render_widget(Clear, area);

        match &self.screen {
            Screen::Welcome => self.draw_welcome(frame, area),
            Screen::NewIdentity => self.draw_new_identity(frame, area),
            Screen::JoinInput => self.draw_join_input(frame, area),
            Screen::JoinSyncing => self.draw_syncing(frame, area),
            Screen::Complete => {} // Will exit immediately
        }
    }

    fn draw_welcome(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(10),
                Constraint::Length(3),
            ])
            .margin(2)
            .split(area);

        // Title
        let title = Paragraph::new(vec![
            Line::from(vec![Span::styled(
                "Welcome to ROTT",
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Cyan),
            )]),
            Line::from(""),
            Line::from("Read Over The Top - Local-first link & note management"),
        ])
        .alignment(ratatui::layout::Alignment::Center);
        frame.render_widget(title, chunks[0]);

        // Options
        let content = Paragraph::new(vec![
            Line::from(""),
            Line::from("Is this your first device with ROTT?"),
            Line::from(""),
            Line::from(vec![
                Span::styled("  [1] ", Style::default().fg(Color::Yellow)),
                Span::raw("Yes, create new identity"),
            ]),
            Line::from(vec![
                Span::styled("      ", Style::default()),
                Span::styled(
                    "Start fresh with a new root document",
                    Style::default().add_modifier(Modifier::DIM),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  [2] ", Style::default().fg(Color::Yellow)),
                Span::raw("No, join existing data"),
            ]),
            Line::from(vec![
                Span::styled("      ", Style::default()),
                Span::styled(
                    "Enter root document ID from another device",
                    Style::default().add_modifier(Modifier::DIM),
                ),
            ]),
        ])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Setup ")
                .border_style(Style::default().fg(Color::Blue)),
        );
        frame.render_widget(content, chunks[1]);

        // Footer
        let footer = if let Some(err) = &self.error {
            Paragraph::new(Span::styled(err, Style::default().fg(Color::Red)))
        } else {
            Paragraph::new(Span::styled(
                "Press 1 or 2 to choose, q to quit",
                Style::default().add_modifier(Modifier::DIM),
            ))
        };
        frame.render_widget(footer, chunks[2]);
    }

    fn draw_new_identity(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2),
                Constraint::Min(12),
                Constraint::Length(3),
            ])
            .margin(2)
            .split(area);

        // Title
        let title = Paragraph::new(vec![Line::from(vec![Span::styled(
            "New Identity Created",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Green),
        )])])
        .alignment(ratatui::layout::Alignment::Center);
        frame.render_widget(title, chunks[0]);

        // ID display
        let id_str = self
            .generated_id
            .map(|id| id.to_string())
            .unwrap_or_default();

        let content = Paragraph::new(vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                "Your Root Document ID:",
                Style::default().add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from(vec![Span::styled(
                format!("  {}", id_str),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from(""),
            Line::from(vec![Span::styled(
                "⚠  IMPORTANT: Save this ID!",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from("You'll need it to set up ROTT on other devices."),
            Line::from("Store it somewhere safe - it cannot be recovered."),
            Line::from(""),
            Line::from(""),
            if self.id_acknowledged {
                Line::from(vec![Span::styled(
                    "✓ Acknowledged. Press Enter to continue.",
                    Style::default().fg(Color::Green),
                )])
            } else {
                Line::from(vec![
                    Span::styled("[y] ", Style::default().fg(Color::Yellow)),
                    Span::raw("Copy to clipboard   "),
                    Span::styled("[Enter] ", Style::default().fg(Color::Yellow)),
                    Span::raw("I've saved it"),
                ])
            },
        ])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Your Identity ")
                .border_style(Style::default().fg(Color::Green)),
        );
        frame.render_widget(content, chunks[1]);

        // Footer/error
        let footer = if let Some(msg) = &self.error {
            let color = if msg.contains("Copied") || msg.contains("saved") {
                Color::Green
            } else {
                Color::Yellow
            };
            Paragraph::new(Span::styled(msg, Style::default().fg(color)))
        } else {
            Paragraph::new(Span::styled(
                "You must acknowledge saving the ID before continuing",
                Style::default().add_modifier(Modifier::DIM),
            ))
        };
        frame.render_widget(footer, chunks[2]);
    }

    fn draw_join_input(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2),
                Constraint::Min(10),
                Constraint::Length(3),
            ])
            .margin(2)
            .split(area);

        // Title
        let title = Paragraph::new(vec![Line::from(vec![Span::styled(
            "Join Existing Data",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Cyan),
        )])])
        .alignment(ratatui::layout::Alignment::Center);
        frame.render_widget(title, chunks[0]);

        // Input area
        let content = Paragraph::new(vec![
            Line::from(""),
            Line::from("Enter your root document ID from another device:"),
            Line::from(""),
            Line::from(vec![
                Span::styled("> ", Style::default().fg(Color::Yellow)),
                Span::raw(&self.input),
                Span::styled("█", Style::default().add_modifier(Modifier::SLOW_BLINK)),
            ]),
            Line::from(""),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Tip: Use Ctrl+V to paste",
                Style::default().add_modifier(Modifier::DIM),
            )]),
        ])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Enter ID ")
                .border_style(Style::default().fg(Color::Blue)),
        );
        frame.render_widget(content, chunks[1]);

        // Footer/error
        let footer = if let Some(err) = &self.error {
            Paragraph::new(Span::styled(err, Style::default().fg(Color::Red)))
        } else {
            Paragraph::new(Span::styled(
                "Press Enter to connect, Esc to go back",
                Style::default().add_modifier(Modifier::DIM),
            ))
        };
        frame.render_widget(footer, chunks[2]);

        // Set cursor position
        let input_x = chunks[1].x + 3 + self.cursor as u16;
        let input_y = chunks[1].y + 4;
        frame.set_cursor_position((input_x, input_y));
    }

    fn draw_syncing(&self, frame: &mut Frame, area: Rect) {
        let popup_area = centered_rect(50, 30, area);
        frame.render_widget(Clear, popup_area);

        let content = Paragraph::new(vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                "↻ Syncing...",
                Style::default().fg(Color::Yellow),
            )]),
            Line::from(""),
            Line::from("Connecting to sync server and pulling your data."),
            Line::from("This may take a moment."),
        ])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Sync in Progress ")
                .border_style(Style::default().fg(Color::Yellow)),
        )
        .alignment(ratatui::layout::Alignment::Center)
        .wrap(Wrap { trim: true });

        frame.render_widget(content, popup_area);
    }
}

/// Copy text to clipboard (platform-specific)
pub fn copy_to_clipboard(text: &str) -> bool {
    // Try using external clipboard tools
    #[cfg(target_os = "linux")]
    {
        use std::io::Write;
        use std::process::{Command, Stdio};

        // Try xclip first, then xsel
        for cmd in &["xclip", "xsel"] {
            let args = if *cmd == "xclip" {
                vec!["-selection", "clipboard"]
            } else {
                vec!["--clipboard", "--input"]
            };

            if let Ok(mut child) = Command::new(cmd)
                .args(&args)
                .stdin(Stdio::piped())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
            {
                if let Some(mut stdin) = child.stdin.take() {
                    if stdin.write_all(text.as_bytes()).is_ok() {
                        let _ = stdin.flush();
                        drop(stdin);
                        if child.wait().map(|s| s.success()).unwrap_or(false) {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    #[cfg(target_os = "macos")]
    {
        use std::io::Write;
        use std::process::{Command, Stdio};

        if let Ok(mut child) = Command::new("pbcopy")
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
        {
            if let Some(mut stdin) = child.stdin.take() {
                if stdin.write_all(text.as_bytes()).is_ok() {
                    let _ = stdin.flush();
                    drop(stdin);
                    return child.wait().map(|s| s.success()).unwrap_or(false);
                }
            }
        }
        false
    }

    #[cfg(target_os = "windows")]
    {
        // Windows clipboard handling via clip.exe
        use std::io::Write;
        use std::process::{Command, Stdio};

        if let Ok(mut child) = Command::new("clip")
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
        {
            if let Some(mut stdin) = child.stdin.take() {
                if stdin.write_all(text.as_bytes()).is_ok() {
                    let _ = stdin.flush();
                    drop(stdin);
                    return child.wait().map(|s| s.success()).unwrap_or(false);
                }
            }
        }
        false
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        let _ = text;
        false
    }
}

/// Paste from clipboard (platform-specific)
fn paste_from_clipboard() -> Option<String> {
    #[cfg(target_os = "linux")]
    {
        use std::process::Command;

        // Try xclip first, then xsel
        for (cmd, args) in &[
            ("xclip", vec!["-selection", "clipboard", "-o"]),
            ("xsel", vec!["--clipboard", "--output"]),
        ] {
            if let Ok(output) = Command::new(cmd).args(args).output() {
                if output.status.success() {
                    return String::from_utf8(output.stdout).ok();
                }
            }
        }
        None
    }

    #[cfg(target_os = "macos")]
    {
        use std::process::Command;

        if let Ok(output) = Command::new("pbpaste").output() {
            if output.status.success() {
                return String::from_utf8(output.stdout).ok();
            }
        }
        None
    }

    #[cfg(target_os = "windows")]
    {
        use std::process::Command;

        // PowerShell method for clipboard read
        if let Ok(output) = Command::new("powershell")
            .args(["-command", "Get-Clipboard"])
            .output()
        {
            if output.status.success() {
                return String::from_utf8(output.stdout).ok();
            }
        }
        None
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        None
    }
}

/// Helper to create a centered rect
fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

// Re-export clipboard functions for use in device panel
pub use copy_to_clipboard as clipboard_copy;
