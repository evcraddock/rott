use std::collections::HashSet;

use ratatui::widgets::ListState;

use crate::links::{Link, LinkService};

pub struct App {
    pub topics: StatefulList<String>,
    pub pages: StatefulList<Link>,
    pub active_pane: ActivePane,
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

impl<T> StatefulList<T> {
    fn new(items: Vec<T>) -> StatefulList<T> {
        StatefulList {
            state: ListState::default(),
            items,
        }
    }

    pub fn next(&mut self) {
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
    pub fn new(selected_topic: Option<String>) -> App {
        let svc = LinkService::new();
        let links = svc
            .load_from_directory("/Users/erik/files/Notes/Inbox")
            .unwrap();

        let mut all_tags = HashSet::new();
        let mut link_titles = Vec::new();
        for link in links.clone() {
            all_tags.extend(link.tags.clone());
        }
        let topics: Vec<String> = all_tags.into_iter().collect();

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
        }
    }

    pub fn reload(&mut self, selected_topic: String) {
        let svc = LinkService::new();
        let links = svc
            .load_from_directory("/Users/erik/files/Notes/Inbox")
            .unwrap();

        let link_titles = links
            .into_iter()
            .filter(|link| link.tags.contains(&selected_topic))
            .collect();
        self.pages = StatefulList::new(link_titles);
    }
}
