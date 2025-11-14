use crossbeam_channel::{bounded, Receiver, Sender};
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub enum FileEvent {
    Changed,
}

pub struct FileWatcher {
    _watcher: Arc<Mutex<RecommendedWatcher>>,
    pub receiver: Receiver<FileEvent>,
}

impl FileWatcher {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let (tx, rx) = bounded::<FileEvent>(100);

        let watcher = create_watcher(tx, path)?;

        Ok(FileWatcher {
            _watcher: Arc::new(Mutex::new(watcher)),
            receiver: rx,
        })
    }

    pub fn check_events(&self) -> bool {
        // Drain all pending events and return true if any exist
        let mut has_events = false;
        while self.receiver.try_recv().is_ok() {
            has_events = true;
        }
        has_events
    }
}

fn create_watcher<P: AsRef<Path>>(
    tx: Sender<FileEvent>,
    path: P,
) -> Result<RecommendedWatcher, Box<dyn std::error::Error>> {
    let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
        match res {
            Ok(event) => {
                // Only send events for file modifications, creations, and deletions
                match event.kind {
                    EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => {
                        // Filter for .md files
                        let is_markdown = event.paths.iter().any(|p| {
                            p.extension().map_or(false, |ext| ext == "md")
                        });

                        if is_markdown {
                            let _ = tx.send(FileEvent::Changed);
                        }
                    }
                    _ => {}
                }
            }
            Err(e) => eprintln!("Watch error: {:?}", e),
        }
    })?;

    // Expand ~ to home directory if needed
    let expanded_path = if path.as_ref().to_string_lossy().starts_with("~/") {
        if let Some(home) = dirs::home_dir() {
            home.join(&path.as_ref().to_string_lossy()[2..])
        } else {
            path.as_ref().to_path_buf()
        }
    } else {
        path.as_ref().to_path_buf()
    };

    watcher.watch(&expanded_path, RecursiveMode::NonRecursive)?;

    Ok(watcher)
}
