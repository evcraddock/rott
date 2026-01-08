//! Application state and logic

use rott_core::{Link, Store};

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
    /// Current list of links (filtered)
    pub links: Vec<Link>,
    /// Currently selected link index
    pub link_index: usize,
    /// Status message to display temporarily
    pub status_message: Option<String>,
}

impl App {
    /// Create a new app with data from store
    pub fn new(store: &Store) -> anyhow::Result<Self> {
        let all_tags = store.get_all_tags()?;
        let links = store.get_all_links()?;

        // Build initial filters list
        let mut filters = vec![Filter::Favorites, Filter::Recent, Filter::Untagged];
        if !all_tags.is_empty() {
            filters.push(Filter::TagsHeader);
        }

        Ok(Self {
            should_quit: false,
            active_pane: ActivePane::Items,
            filters,
            filter_index: 1, // Start on "Recent"
            tags_expanded: false,
            all_tags,
            links,
            link_index: 0,
            status_message: None,
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
                }
            }
            ActivePane::Detail => {
                // Could scroll detail view in the future
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
                }
            }
            ActivePane::Detail => {
                // Could scroll detail view in the future
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
                }
            }
            ActivePane::Items => {
                // Open link in browser
                if let Some(link) = self.current_link() {
                    if let Err(e) = open::that(&link.url) {
                        self.status_message = Some(format!("Failed to open: {}", e));
                    } else {
                        self.status_message = Some("Opened in browser".to_string());
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
                // TODO: Get favorite tag from config
                // For now, show all links
                store.get_all_links()?
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

        // Reset link selection
        self.link_index = 0;

        Ok(())
    }

    /// Refresh data from store
    #[allow(dead_code)]
    pub fn refresh(&mut self, store: &Store) -> anyhow::Result<()> {
        self.all_tags = store.get_all_tags()?;
        self.rebuild_filters();
        self.apply_filter(store)?;
        Ok(())
    }
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
}
