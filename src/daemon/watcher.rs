use crate::core::config::Config;
use crate::core::indexer::IndexWorker;
use crate::core::languages::SupportedLanguage;
use notify::event::{ModifyKind, RenameMode};
use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher, Config as NotifyConfig};
use std::path::Path;
use std::sync::mpsc::channel;
use std::thread;
use tracing::{debug, error, info};

pub struct FileWatcher;

pub enum WatcherEvent {
    Update(String),
    Remove(String),
}

impl FileWatcher {
    pub async fn watch(root: &str) -> anyhow::Result<()> {
        let config = Config::load(root);
        let excludes = config.ignore_patterns;

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

        let mut watcher = RecommendedWatcher::new(tx, NotifyConfig::default())?;
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

                            let is_old_excluded = excludes.iter().any(|p| old_path.contains(p));
                            let is_new_excluded = excludes.iter().any(|p| new_path.contains(p));

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

                        if excludes.iter().any(|p| path_str.contains(p)) {
                            continue;
                        }

                        if SupportedLanguage::from_path(&path).is_some() {
                            match kind {
                                EventKind::Create(_) | EventKind::Modify(_) => {
                                    let _ = worker_tx.send(WatcherEvent::Update(path_str));
                                }
                                EventKind::Remove(_) => {
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