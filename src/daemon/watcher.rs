use crate::core::config::Config;
use crate::core::indexer::IndexWorker;
use crate::core::languages::SupportedLanguage;
use crossbeam_channel::{bounded, select, tick};
use notify::event::{ModifyKind, RenameMode};
use notify::{Config as NotifyConfig, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::path::Path;
use std::sync::mpsc::{channel, TryRecvError};
use std::thread;
use std::time::{Duration, Instant};
use tracing::{error, info, warn};

pub struct FileWatcher;

pub enum WatcherEvent {
    Update(String),
    Remove(String),
}

#[derive(Clone, Copy)]
enum PendingKind {
    Update,
    Remove,
}

impl FileWatcher {
    pub async fn watch(root: &str) -> anyhow::Result<()> {
        let config = Config::load(root);
        let excludes = config.ignore_patterns;

        let (tx, rx) = channel();
        let (worker_tx, worker_rx) = bounded::<WatcherEvent>(512);
        let root_clone = root.to_string();

        thread::spawn(move || match IndexWorker::new(&root_clone) {
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
        });

        let mut watcher = RecommendedWatcher::new(tx, NotifyConfig::default())?;
        watcher.watch(Path::new(root), RecursiveMode::Recursive)?;

        info!("Watcher started on: {}", root);

        let debounce_duration = Duration::from_millis(300);
        let mut pending: HashMap<String, (PendingKind, Instant)> = HashMap::new();
        let ticker = tick(Duration::from_millis(100));

        loop {
            select! {
                recv(ticker) -> _ => {
                    let now = Instant::now();
                    let ready_keys: Vec<String> = pending
                        .iter()
                        .filter(|(_, (_, ts))| now.duration_since(*ts) >= debounce_duration)
                        .map(|(k, _)| k.clone())
                        .collect();

                    for key in ready_keys {
                        if let Some((kind, _)) = pending.remove(&key) {
                            let event = match kind {
                                PendingKind::Update => WatcherEvent::Update(key.clone()),
                                PendingKind::Remove => WatcherEvent::Remove(key.clone()),
                            };
                            if worker_tx.is_full() {
                                warn!("Worker queue full, dropping event for: {}", key);
                                continue;
                            }
                            if let Err(e) = worker_tx.send(event) {
                                error!("Failed to send to worker: {}", e);
                            }
                        }
                    }
                }
            }

            loop {
                match rx.try_recv() {
                    Ok(Ok(event)) => {
                        let kind = event.kind;

                        if let EventKind::Modify(ModifyKind::Name(RenameMode::Both)) = kind {
                            if event.paths.len() == 2 {
                                let old_path = event.paths[0].to_string_lossy().into_owned();
                                let new_path = event.paths[1].to_string_lossy().into_owned();

                                if !excludes.iter().any(|p| old_path.contains(p)) {
                                    pending.insert(
                                        old_path.clone(),
                                        (PendingKind::Remove, Instant::now()),
                                    );
                                }
                                if !excludes.iter().any(|p| new_path.contains(p)) {
                                    pending.insert(
                                        new_path.clone(),
                                        (PendingKind::Update, Instant::now()),
                                    );
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
                                let pkind = match kind {
                                    EventKind::Create(_) | EventKind::Modify(_) => {
                                        PendingKind::Update
                                    }
                                    EventKind::Remove(_) => PendingKind::Remove,
                                    _ => continue,
                                };
                                pending.insert(path_str, (pkind, Instant::now()));
                            }
                        }
                    }
                    Ok(Err(e)) => error!("Watch error: {:?}", e),
                    Err(TryRecvError::Empty) => break,
                    Err(TryRecvError::Disconnected) => return Ok(()),
                }
            }
        }
    }
}
