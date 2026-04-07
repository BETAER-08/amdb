use crate::core::indexer::IndexWorker;
use crate::core::languages::SupportedLanguage;
use notify::event::{ModifyKind, RenameMode};
use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::mpsc::channel;
use std::thread;
use tracing::{debug, error, info};

pub struct FileWatcher;

pub enum WatcherEvent {
    Update(String),
    Remove(String),
}

const DEFAULT_EXCLUDES: &[&str] = &[
    "target", ".git", "node_modules", ".amdb", ".fastembed_cache", "__pycache__", ".database"
];

impl FileWatcher {
    pub async fn watch(root: &str) -> anyhow::Result<()> {
        let (tx, rx) = channel();
        let (worker_tx, worker_rx) = channel::<WatcherEvent>();
        let root_clone = root.to_string();

        thread::spawn(move || {
            match IndexWorker::new(&root_clone) {
                Ok(mut worker) => {
                    info!("Worker thread initialized successfully.");
                    for event in worker_rx {
                        match event {
                            WatcherEvent::Update(path) => {
                                if let Err(e) = worker.update_file(&path) {
                                    error!("Worker update error: {}", e);
                                }
                            }
                            WatcherEvent::Remove(path) => {
                                if let Err(e) = worker.remove_file(&path) {
                                    error!("Worker remove error: {}", e);
                                }
                            }
                        }
                    }
                }
                Err(e) => error!("Failed to initialize worker thread: {}", e),
            }
        });

        let mut watcher = RecommendedWatcher::new(tx, Config::default())?;
        watcher.watch(Path::new(root), RecursiveMode::Recursive)?;

        info!("Watcher started on: {}", root);

        for res in rx {
            match res {
                Ok(event) => {
                    let kind = event.kind;

                    if let EventKind::Modify(ModifyKind::Name(RenameMode::Both)) = kind {
                        if event.paths.len() == 2 {
                            let old_path = event.paths[0].to_string_lossy().into_owned();
                            let new_path = event.paths[1].to_string_lossy().into_owned();

                            debug!("Rename detected: {} -> {}", old_path, new_path);

                            let is_old_excluded = DEFAULT_EXCLUDES.iter().any(|p| old_path.contains(p));
                            let is_new_excluded = DEFAULT_EXCLUDES.iter().any(|p| new_path.contains(p));

                            if !is_old_excluded {
                                let _ = worker_tx.send(WatcherEvent::Remove(old_path));
                            }

                            if !is_new_excluded {
                                let _ = worker_tx.send(WatcherEvent::Update(new_path));
                            }
                            continue;
                        }
                    }

                    for path in event.paths {
                        let path_str = path.to_string_lossy().into_owned();

                        if DEFAULT_EXCLUDES.iter().any(|p| path_str.contains(p)) {
                            continue;
                        }

                        if SupportedLanguage::from_path(&path).is_some() {
                            match kind {
                                EventKind::Create(_) | EventKind::Modify(_) => {
                                    debug!("Detected change in: {}", path_str);
                                    let _ = worker_tx.send(WatcherEvent::Update(path_str));
                                }
                                EventKind::Remove(_) => {
                                    debug!("Detected removal of: {}", path_str);
                                    let _ = worker_tx.send(WatcherEvent::Remove(path_str));
                                }
                                _ => {}
                            }
                        }
                    }
                }
                Err(e) => error!("Watch error: {:?}", e),
            }
        }
        Ok(())
    }
}