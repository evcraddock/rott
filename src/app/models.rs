use ratatui::widgets::ListState;

pub struct App {
    pub topics: StatefulList<String>,
    pub pages: StatefulList<String>,
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
    pub fn new() -> App {
        // Mock data
        let topics = vec![
            "Programming".to_string(),
            "Science".to_string(),
            "Technology".to_string(),
            "Arts".to_string(),
            "History".to_string(),
        ];

        let pages = vec![
            "Introduction to Rust Programming".to_string(),
            "The Future of AI".to_string(),
            "Web Development in 2025".to_string(),
            "Modern Art Movements".to_string(),
            "Ancient Civilizations".to_string(),
        ];

        App {
            topics: StatefulList::new(topics),
            pages: StatefulList::new(pages),
            active_pane: ActivePane::Topics,
        }
    }
}
