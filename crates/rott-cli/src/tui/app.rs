//! Application state and logic

use rott_core::{Link, Note, Store};
use std::process::{Command, Stdio};

// Re-export UrlMetadata from crate's metadata module
pub use crate::metadata::UrlMetadata;

/// Input mode for the application
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    /// Normal navigation mode
    Normal,
    /// Command input mode (after pressing : or command key)
    Command,
    /// Filter/search mode (after pressing /)
    Filter,
}

/// Type of command being entered
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandType {
    /// Generic command starting with :
    Generic,
    /// Add a new link
    Add,
    /// Edit tags on selected link
    Tag,
    /// Add note to selected link
    Note,
    /// Edit selected link
    Edit,
}

/// Which pane has focus
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivePane {
    Filters,
    Items,
    Detail,
}

impl ActivePane {
    /// Move to the next pane (wrapping)
    pub fn next(self) -> Self {
        match self {
            ActivePane::Filters => ActivePane::Items,
            ActivePane::Items => ActivePane::Detail,
            ActivePane::Detail => ActivePane::Filters,
        }
    }

    /// Move to the previous pane (wrapping)
    pub fn prev(self) -> Self {
        match self {
            ActivePane::Filters => ActivePane::Detail,
            ActivePane::Items => ActivePane::Filters,
            ActivePane::Detail => ActivePane::Items,
        }
    }
}

/// Smart filter options in the left pane
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Filter {
    Favorites,
    Recent,
    Untagged,
    /// The "By Tag..." accordion header
    TagsHeader,
    /// An individual tag filter
    ByTag(String),
}

/// Application state
pub struct App {
    /// Whether the app should exit
    pub should_quit: bool,
    /// Current input mode
    pub input_mode: InputMode,
    /// Type of command being entered
    pub command_type: Option<CommandType>,
    /// Command input buffer
    pub command_input: String,
    /// Cursor position in command input
    pub command_cursor: usize,
    /// Which pane has focus
    pub active_pane: ActivePane,
    /// Available filters (includes expanded tags)
    pub filters: Vec<Filter>,
    /// Currently selected filter index
    pub filter_index: usize,
    /// Whether the "By Tag..." accordion is expanded
    pub tags_expanded: bool,
    /// All available tags
    pub all_tags: Vec<String>,
    /// All links (unfiltered, for search)
    pub all_links: Vec<Link>,
    /// Current list of links (filtered)
    pub links: Vec<Link>,
    /// Currently selected link index
    pub link_index: usize,
    /// Status message to display temporarily
    pub status_message: Option<String>,
    /// Last deleted link (for undo)
    pub deleted_link: Option<Link>,
    /// Filter text for real-time filtering
    pub filter_text: String,
    /// Whether we're currently adding a link (async operation)
    pub is_loading: bool,
    /// Scroll offset for detail pane
    pub detail_scroll: u16,
    /// When the status message was set (for auto-dismiss)
    pub status_message_time: Option<std::time::Instant>,
    /// Whether help overlay is visible
    pub show_help: bool,
    /// Sync status indicator
    pub sync_status: SyncIndicator,
    /// Pending 'g' keypress for gg sequence (with timestamp)
    pub pending_g: Option<std::time::Instant>,
}

/// Sync status indicator
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncIndicator {
    /// Connected and synced
    Synced,
    /// Sync in progress
    Syncing,
    /// Disconnected, will retry
    Offline,
    /// Sync not configured
    Disabled,
    /// Sync error occurred
    Error,
}

impl App {
    /// Create a new app with data from store
    pub fn new(store: &Store) -> anyhow::Result<Self> {
        let all_tags = store.get_all_tags()?;
        let all_links = store.get_all_links()?;
        let links = all_links.clone();

        // Build initial filters list
        let mut filters = vec![Filter::Favorites, Filter::Recent, Filter::Untagged];
        if !all_tags.is_empty() {
            filters.push(Filter::TagsHeader);
        }

        Ok(Self {
            should_quit: false,
            input_mode: InputMode::Normal,
            command_type: None,
            command_input: String::new(),
            command_cursor: 0,
            active_pane: ActivePane::Items,
            filters,
            filter_index: 0, // Start on "Favorites"
            tags_expanded: false,
            all_tags,
            all_links,
            links,
            link_index: 0,
            status_message: None,
            deleted_link: None,
            filter_text: String::new(),
            is_loading: false,
            detail_scroll: 0,
            status_message_time: None,
            show_help: false,
            sync_status: if store.config().sync_enabled {
                SyncIndicator::Syncing
            } else {
                SyncIndicator::Disabled
            },
            pending_g: None,
        })
    }

    /// Rebuild filters list based on expanded state
    fn rebuild_filters(&mut self) {
        let mut filters = vec![Filter::Favorites, Filter::Recent, Filter::Untagged];

        // Only show "By Tag..." if there are tags
        if !self.all_tags.is_empty() {
            filters.push(Filter::TagsHeader);

            if self.tags_expanded {
                for tag in &self.all_tags {
                    filters.push(Filter::ByTag(tag.clone()));
                }
            }
        }

        self.filters = filters;
    }

    /// Toggle the tags accordion
    pub fn toggle_tags_accordion(&mut self) {
        self.tags_expanded = !self.tags_expanded;
        self.rebuild_filters();
    }

    /// Get the currently selected filter
    pub fn current_filter(&self) -> Option<&Filter> {
        self.filters.get(self.filter_index)
    }

    /// Set a status message (will auto-dismiss after 3 seconds)
    pub fn set_status(&mut self, message: impl Into<String>) {
        self.status_message = Some(message.into());
        self.status_message_time = Some(std::time::Instant::now());
    }

    /// Check and clear expired status message
    pub fn check_status_timeout(&mut self) {
        if let Some(time) = self.status_message_time {
            if time.elapsed() > std::time::Duration::from_secs(3) {
                self.status_message = None;
                self.status_message_time = None;
            }
        }
    }

    /// Toggle help overlay
    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }

    /// Get the currently selected link
    pub fn current_link(&self) -> Option<&Link> {
        self.links.get(self.link_index)
    }

    /// Move selection up in the current pane
    pub fn move_up(&mut self) {
        match self.active_pane {
            ActivePane::Filters => {
                if self.filter_index > 0 {
                    self.filter_index -= 1;
                }
            }
            ActivePane::Items => {
                if self.link_index > 0 {
                    self.link_index -= 1;
                    self.detail_scroll = 0; // Reset scroll when changing selection
                }
            }
            ActivePane::Detail => {
                // Scroll detail view up
                self.detail_scroll = self.detail_scroll.saturating_sub(1);
            }
        }
    }

    /// Move selection down in the current pane
    pub fn move_down(&mut self) {
        match self.active_pane {
            ActivePane::Filters => {
                if self.filter_index < self.filters.len().saturating_sub(1) {
                    self.filter_index += 1;
                }
            }
            ActivePane::Items => {
                if self.link_index < self.links.len().saturating_sub(1) {
                    self.link_index += 1;
                    self.detail_scroll = 0; // Reset scroll when changing selection
                }
            }
            ActivePane::Detail => {
                // Scroll detail view down
                self.detail_scroll = self.detail_scroll.saturating_add(1);
            }
        }
    }

    /// Move selection to first item in the current pane (vim 'gg')
    pub fn move_to_first(&mut self) {
        match self.active_pane {
            ActivePane::Filters => {
                self.filter_index = 0;
            }
            ActivePane::Items => {
                self.link_index = 0;
                self.detail_scroll = 0;
            }
            ActivePane::Detail => {
                self.detail_scroll = 0;
            }
        }
    }

    /// Move selection to last item in the current pane (vim 'G')
    pub fn move_to_last(&mut self) {
        match self.active_pane {
            ActivePane::Filters => {
                self.filter_index = self.filters.len().saturating_sub(1);
            }
            ActivePane::Items => {
                self.link_index = self.links.len().saturating_sub(1);
                self.detail_scroll = 0;
            }
            ActivePane::Detail => {
                // For detail pane, we can't easily know max scroll, so just add a large value
                // The UI will clamp it appropriately
                self.detail_scroll = u16::MAX;
            }
        }
    }

    /// Move focus to the next pane
    pub fn next_pane(&mut self) {
        self.active_pane = self.active_pane.next();
    }

    /// Move focus to the previous pane
    pub fn prev_pane(&mut self) {
        self.active_pane = self.active_pane.prev();
    }

    /// Handle Enter key in current pane
    pub fn handle_enter(&mut self, store: &Store) -> anyhow::Result<()> {
        match self.active_pane {
            ActivePane::Filters => {
                // Check if we're on the TagsHeader
                if let Some(Filter::TagsHeader) = self.current_filter() {
                    self.toggle_tags_accordion();
                } else {
                    self.apply_filter(store)?;
                    // Auto-switch to Items pane after selecting a filter
                    self.active_pane = ActivePane::Items;
                }
            }
            ActivePane::Items => {
                // Open link in browser
                if let Some(link) = self.current_link() {
                    let url = link.url.clone();
                    let title = link.title.clone();
                    match open_url(&url) {
                        Ok(_) => {
                            self.set_status(format!("Opened '{}'", title));
                        }
                        Err(e) => {
                            self.set_status(format!("Failed to open: {}", e));
                        }
                    }
                }
            }
            ActivePane::Detail => {
                // Could expand notes or similar
            }
        }
        Ok(())
    }

    /// Apply the currently selected filter
    pub fn apply_filter(&mut self, store: &Store) -> anyhow::Result<()> {
        let filter = self.current_filter().cloned();

        self.links = match filter {
            Some(Filter::Favorites) => {
                if let Some(tag) = &store.config().favorite_tag {
                    store.get_links_by_tag(tag)?
                } else {
                    // No favorite tag configured, show empty
                    Vec::new()
                }
            }
            Some(Filter::Recent) => {
                let mut links = store.get_all_links()?;
                links.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
                links
            }
            Some(Filter::Untagged) => {
                let all_links = store.get_all_links()?;
                all_links
                    .into_iter()
                    .filter(|l| l.tags.is_empty())
                    .collect()
            }
            Some(Filter::TagsHeader) => {
                // TagsHeader doesn't filter, just toggles accordion
                return Ok(());
            }
            Some(Filter::ByTag(tag)) => store.get_links_by_tag(&tag)?,
            None => store.get_all_links()?,
        };

        // Clamp link selection to new list bounds (preserve position when possible)
        if self.links.is_empty() {
            self.link_index = 0;
        } else {
            self.link_index = self.link_index.min(self.links.len() - 1);
        }

        Ok(())
    }

    /// Refresh data from store
    pub fn refresh(&mut self, store: &Store) -> anyhow::Result<()> {
        self.all_tags = store.get_all_tags()?;
        self.all_links = store.get_all_links()?;
        self.rebuild_filters();
        self.apply_filter(store)?;
        Ok(())
    }

    /// Enter command mode with a specific command type
    pub fn enter_command_mode(&mut self, cmd_type: CommandType) {
        self.input_mode = InputMode::Command;
        self.command_type = Some(cmd_type.clone());
        self.command_input.clear();
        self.command_cursor = 0;

        // Pre-fill based on command type
        match cmd_type {
            CommandType::Add => {
                self.command_input = "add ".to_string();
                self.command_cursor = 4;
            }
            CommandType::Tag => {
                // Pre-fill with current tags
                if let Some(link) = self.current_link() {
                    self.command_input = format!("tag {}", link.tags.join(", "));
                    self.command_cursor = self.command_input.len();
                } else {
                    self.command_input = "tag ".to_string();
                    self.command_cursor = 4;
                }
            }
            CommandType::Generic => {
                // Just the colon prefix, user types command
            }
            CommandType::Note | CommandType::Edit => {
                // These go directly to editor, no pre-fill needed
            }
        }
    }

    /// Enter filter mode
    pub fn enter_filter_mode(&mut self) {
        self.input_mode = InputMode::Filter;
        self.filter_text.clear();
        self.command_input.clear();
        self.command_cursor = 0;
    }

    /// Exit command/filter mode
    pub fn exit_input_mode(&mut self) {
        self.input_mode = InputMode::Normal;
        self.command_type = None;
        self.command_input.clear();
        self.command_cursor = 0;
    }

    /// Clear filter and show all items
    pub fn clear_filter(&mut self, store: &Store) -> anyhow::Result<()> {
        self.filter_text.clear();
        self.apply_filter(store)?;
        Ok(())
    }

    /// Apply real-time filter to current view
    pub fn apply_realtime_filter(&mut self) {
        if self.filter_text.is_empty() {
            // No filter, show based on current filter selection
            return;
        }

        let filter_lower = self.filter_text.to_lowercase();
        self.links = self
            .all_links
            .iter()
            .filter(|link| {
                link.title.to_lowercase().contains(&filter_lower)
                    || link.url.to_lowercase().contains(&filter_lower)
                    || link
                        .tags
                        .iter()
                        .any(|t| t.to_lowercase().contains(&filter_lower))
            })
            .cloned()
            .collect();

        // Reset selection if out of bounds
        if self.link_index >= self.links.len() {
            self.link_index = 0;
        }
    }

    /// Insert character at cursor position
    pub fn insert_char(&mut self, c: char) {
        self.command_input.insert(self.command_cursor, c);
        self.command_cursor += 1;

        // Update filter in real-time if in filter mode
        if self.input_mode == InputMode::Filter {
            self.filter_text = self.command_input.clone();
            self.apply_realtime_filter();
        }
    }

    /// Delete character before cursor
    pub fn delete_char(&mut self) {
        if self.command_cursor > 0 {
            self.command_cursor -= 1;
            self.command_input.remove(self.command_cursor);

            // Update filter in real-time if in filter mode
            if self.input_mode == InputMode::Filter {
                self.filter_text = self.command_input.clone();
                self.apply_realtime_filter();
            }
        }
    }

    /// Move cursor left
    pub fn cursor_left(&mut self) {
        if self.command_cursor > 0 {
            self.command_cursor -= 1;
        }
    }

    /// Move cursor right
    pub fn cursor_right(&mut self) {
        if self.command_cursor < self.command_input.len() {
            self.command_cursor += 1;
        }
    }

    /// Delete a link and store for undo
    pub fn delete_current_link(&mut self, store: &mut Store) -> anyhow::Result<()> {
        if let Some(link) = self.current_link().cloned() {
            let saved_index = self.link_index;
            store.delete_link(link.id)?;
            self.deleted_link = Some(link.clone());
            self.set_status(format!("Deleted '{}'. Press u to undo", link.title));
            self.refresh(store)?;
            // Restore index, clamped to new list bounds
            if !self.links.is_empty() {
                self.link_index = saved_index.min(self.links.len() - 1);
            }
        }
        Ok(())
    }

    /// Undo last delete
    pub fn undo_delete(&mut self, store: &mut Store) -> anyhow::Result<()> {
        if let Some(link) = self.deleted_link.take() {
            store.add_link(&link)?;
            self.set_status(format!("Restored '{}'", link.title));
            self.refresh(store)?;
        } else {
            self.set_status("Nothing to undo".to_string());
        }
        Ok(())
    }

    /// Add a new link with the given URL
    pub fn add_link(
        &mut self,
        store: &mut Store,
        url: &str,
        metadata: Option<UrlMetadata>,
    ) -> anyhow::Result<()> {
        let mut link = Link::new(url);

        if let Some(meta) = metadata {
            if let Some(title) = meta.title {
                link.set_title(title);
            }
            if let Some(desc) = meta.description {
                link.set_description(Some(desc));
            }
            if !meta.author.is_empty() {
                link.set_author(meta.author);
            }
        }

        store.add_link(&link)?;
        self.set_status(format!("Added '{}'", link.title));
        self.refresh(store)?;
        Ok(())
    }

    /// Update tags on the current link
    pub fn update_tags(&mut self, store: &mut Store, tags_str: &str) -> anyhow::Result<()> {
        if let Some(link) = self.current_link().cloned() {
            let mut updated_link = link;
            let tags: Vec<String> = tags_str
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            updated_link.set_tags(tags);
            store.update_link(&updated_link)?;
            self.set_status("Tags updated".to_string());
            self.refresh(store)?;
        }
        Ok(())
    }

    /// Add a note to the current link
    pub fn add_note_to_current(&mut self, store: &mut Store, body: &str) -> anyhow::Result<()> {
        if let Some(link) = self.current_link() {
            let note = Note::new(body);
            store.add_note_to_link(link.id, &note)?;
            self.set_status("Note added".to_string());
            self.refresh(store)?;
        }
        Ok(())
    }

    /// Search all links
    pub fn search(&mut self, store: &Store, query: &str) -> anyhow::Result<()> {
        if query.is_empty() {
            self.apply_filter(store)?;
        } else {
            self.links = store.search_links(query)?;
            self.link_index = 0;
            self.set_status(format!("Found {} results", self.links.len()));
        }
        Ok(())
    }

    /// Parse and execute command from input
    pub fn execute_command(&mut self, store: &mut Store) -> anyhow::Result<CommandResult> {
        let input = self.command_input.trim().to_string();

        // Parse command
        if input.starts_with("add ") {
            let url = input.strip_prefix("add ").unwrap().trim();
            if url.is_empty() {
                self.set_status("Usage: add <url>".to_string());
                return Ok(CommandResult::Done);
            }
            return Ok(CommandResult::NeedMetadata(url.to_string()));
        } else if input.starts_with("tag ") {
            let tags = input.strip_prefix("tag ").unwrap().trim();
            self.update_tags(store, tags)?;
        } else if input == "note" || input.starts_with("note ") {
            return Ok(CommandResult::NeedEditor(EditorTask::Note));
        } else if input == "edit" {
            return Ok(CommandResult::NeedEditor(EditorTask::EditLink));
        } else if input == "delete" || input == "d" {
            self.delete_current_link(store)?;
        } else if input.starts_with("search ") {
            let query = input.strip_prefix("search ").unwrap().trim();
            self.search(store, query)?;
        } else if !input.is_empty() {
            self.set_status(format!("Unknown command: {}", input));
        }

        Ok(CommandResult::Done)
    }
}

/// Result of command execution
#[derive(Debug)]
pub enum CommandResult {
    /// Command completed
    Done,
    /// Need to fetch metadata for URL
    NeedMetadata(String),
    /// Need to open editor
    NeedEditor(EditorTask),
}

/// Type of editor task
#[derive(Debug)]
pub enum EditorTask {
    /// Add/edit a note
    Note,
    /// Edit link details
    EditLink,
}

/// Open a URL in the default browser
///
/// Uses xdg-open on Linux, open on macOS, start on Windows.
/// Spawns as a detached process with null stdio to avoid
/// interfering with the TUI.
fn open_url(url: &str) -> std::io::Result<()> {
    #[cfg(target_os = "linux")]
    let mut cmd = Command::new("xdg-open");

    #[cfg(target_os = "macos")]
    let mut cmd = Command::new("open");

    #[cfg(target_os = "windows")]
    let mut cmd = {
        let mut c = Command::new("cmd");
        c.args(["/C", "start", ""]);
        c
    };

    cmd.arg(url)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_active_pane_next() {
        assert_eq!(ActivePane::Filters.next(), ActivePane::Items);
        assert_eq!(ActivePane::Items.next(), ActivePane::Detail);
        assert_eq!(ActivePane::Detail.next(), ActivePane::Filters);
    }

    #[test]
    fn test_active_pane_prev() {
        assert_eq!(ActivePane::Filters.prev(), ActivePane::Detail);
        assert_eq!(ActivePane::Items.prev(), ActivePane::Filters);
        assert_eq!(ActivePane::Detail.prev(), ActivePane::Items);
    }

    #[test]
    fn test_filter_variants() {
        let fav = Filter::Favorites;
        let recent = Filter::Recent;
        let untagged = Filter::Untagged;
        let by_tag = Filter::ByTag("rust".to_string());

        assert_eq!(fav, Filter::Favorites);
        assert_eq!(recent, Filter::Recent);
        assert_eq!(untagged, Filter::Untagged);
        assert_eq!(by_tag, Filter::ByTag("rust".to_string()));
    }

    #[test]
    fn test_input_mode() {
        assert_eq!(InputMode::Normal, InputMode::Normal);
        assert_eq!(InputMode::Command, InputMode::Command);
        assert_eq!(InputMode::Filter, InputMode::Filter);
        assert_ne!(InputMode::Normal, InputMode::Command);
    }

    #[test]
    fn test_command_type() {
        assert_eq!(CommandType::Add, CommandType::Add);
        assert_eq!(CommandType::Tag, CommandType::Tag);
        assert_ne!(CommandType::Add, CommandType::Tag);
    }
}
