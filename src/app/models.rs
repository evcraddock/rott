use std::{collections::HashSet, sync::Arc};

use ratatui::widgets::ListState;

use crate::{
    config::AppConfig,
    links::{Link, LinkService},
};

pub struct App {
    pub topics: StatefulList<String>,
    pub pages: StatefulList<Link>,
    pub active_pane: ActivePane,
    pub input_mode: InputMode,
    pub tag_input: String,
    config: Arc<AppConfig>,
}

pub struct StatefulList<T> {
    pub state: ListState,
    pub items: Vec<T>,
}

#[derive(PartialEq)]
pub enum ActivePane {
    Topics,
    Pages,
}

#[derive(PartialEq)]
pub enum InputMode {
    Normal,
    EditingTags,
}

impl<T> StatefulList<T> {
    fn new(items: Vec<T>) -> StatefulList<T> {
        StatefulList {
            state: ListState::default(),
            items,
        }
    }

    pub fn next(&mut self) {
        if self.items.is_empty() {
            self.state.select(None);
            return;
        }

        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len().saturating_sub(1) {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        if self.items.is_empty() {
            self.state.select(None);
            return;
        }

        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len().saturating_sub(1)
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
}

impl App {
    pub fn new(selected_topic: Option<String>, config: &Arc<AppConfig>) -> App {
        let svc = LinkService::new();
        let links = svc.load_from_directory(config.links_path.clone()).unwrap();

        let mut all_tags = HashSet::new();
        let mut link_titles = Vec::new();
        for link in links.clone() {
            all_tags.extend(link.tags.clone());
        }
        let mut topics: Vec<String> = all_tags.into_iter().collect();
        topics.sort();

        let selected_topic = selected_topic.or_else(|| topics.first().cloned());

        if let Some(topic) = selected_topic {
            link_titles = links
                .into_iter()
                .filter(|link| link.tags.contains(&topic))
                .collect();
        }

        App {
            topics: StatefulList::new(topics),
            pages: StatefulList::new(link_titles),
            active_pane: ActivePane::Topics,
            input_mode: InputMode::Normal,
            tag_input: String::new(),
            config: config.clone(),
        }
    }

    pub fn reload(&mut self, selected_topic: String) {
        // Preserve current page selection when reloading
        let current_selection = self.pages.state.selected();
        self.reload_with_page_selection(selected_topic, current_selection);
    }

    pub fn reload_with_page_selection(&mut self, selected_topic: String, page_index: Option<usize>) {
        let svc = LinkService::new();
        let mut links = svc
            .load_from_directory(self.config.links_path.clone())
            .unwrap();

        links.sort_by(|a, b| a.created.cmp(&b.created)); // Sort by oldest first

        // Rebuild topics list from all tags
        let mut all_tags = HashSet::new();
        for link in &links {
            all_tags.extend(link.tags.clone());
        }
        let mut topics: Vec<String> = all_tags.into_iter().collect();
        topics.sort();

        // Find the index of the selected topic in the new topics list
        let selected_index = topics.iter().position(|t| t == &selected_topic);

        // Update topics list and preserve selection
        self.topics = StatefulList::new(topics);
        if let Some(index) = selected_index {
            self.topics.state.select(Some(index));
        }

        // Filter articles by selected topic
        let link_titles = links
            .into_iter()
            .filter(|link| link.tags.contains(&selected_topic))
            .map(|link| link)
            .collect();
        self.pages = StatefulList::new(link_titles);

        // Restore page selection if provided and valid
        if let Some(index) = page_index {
            if index < self.pages.items.len() {
                self.pages.state.select(Some(index));
            } else if !self.pages.items.is_empty() {
                // Index out of bounds, select last item
                self.pages.state.select(Some(self.pages.items.len() - 1));
            }
        }
    }

    pub fn delete_link(&self, link: &Link) {
        let svc = LinkService::new();
        if let Some(file_path) = &link.file_path {
            svc.delete_link(file_path.as_str())
                .expect("could not delete file");
        }
    }

    pub fn move_link_to_drafts(&self, link: &Link) {
        let svc = LinkService::new();
        if let Some(file_path) = &link.file_path {
            // Update tags: remove "readlater" and add "linkblog"
            svc.update_tags(file_path.as_str(), "readlater", "linkblog")
                .expect("could not update tags");

            // Move the file to drafts
            svc.move_link(file_path.as_str(), &self.config.draft_location)
                .expect("could not move file");
        }
    }

    pub fn start_editing_tags(&mut self) {
        if let Some(index) = self.pages.state.selected() {
            if let Some(link) = self.pages.items.get(index) {
                // Load current tags as comma-separated string
                self.tag_input = link.tags.join(", ");
                self.input_mode = InputMode::EditingTags;
            }
        }
    }

    pub fn save_edited_tags(&mut self) {
        if let Some(index) = self.pages.state.selected() {
            if let Some(link) = self.pages.items.get(index) {
                if let Some(file_path) = &link.file_path {
                    let svc = LinkService::new();

                    // Parse new tags from input
                    let new_tags: Vec<String> = self.tag_input
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();

                    // Update tags in file
                    if let Err(e) = svc.replace_all_tags(file_path.as_str(), &new_tags) {
                        eprintln!("Error updating tags: {}", e);
                    }
                }
            }
        }

        // Exit editing mode
        self.input_mode = InputMode::Normal;
        self.tag_input.clear();
    }

    pub fn cancel_editing(&mut self) {
        self.input_mode = InputMode::Normal;
        self.tag_input.clear();
    }
}
