use crate::core::config::AppConfig;
use crate::core::indexer::Indexer;
use crate::core::languages::SupportedLanguage;
use notify::event::{ModifyKind, RenameMode};
use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::mpsc::channel;
use tracing::{debug, error, info};

pub struct FileWatcher;

impl FileWatcher {
    pub async fn watch(root: &str, config: &AppConfig) -> anyhow::Result<()> {
        let (tx, rx) = channel();
        let mut watcher = RecommendedWatcher::new(tx, Config::default())?;

        watcher.watch(Path::new(root), RecursiveMode::Recursive)?;

        info!("Watcher started on: {}", root);

        for res in rx {
            match res {
                Ok(event) => {
                    let kind = event.kind;

                    if let EventKind::Modify(ModifyKind::Name(RenameMode::Both)) = kind {
                        if event.paths.len() == 2 {
                            let old_path = event.paths[0].to_string_lossy();
                            let new_path = event.paths[1].to_string_lossy();

                            debug!("Rename detected: {} -> {}", old_path, new_path);

                            let is_old_excluded = config.exclude_patterns.iter().any(|p| old_path.contains(p));
                            let is_new_excluded = config.exclude_patterns.iter().any(|p| new_path.contains(p));

                            if !is_old_excluded {
                                if let Err(e) = Indexer::remove_file(root, &old_path) {
                                    error!("Remove error: {}", e);
                                }
                            }

                            if !is_new_excluded {
                                if let Err(e) = Indexer::update_file(root, &new_path) {
                                    error!("Update error: {}", e);
                                }
                            }
                            continue;
                        }
                    }

                    for path in event.paths {
                        let path_str = path.to_string_lossy().to_string();

                        if config.exclude_patterns.iter().any(|p| path_str.contains(p)) {
                            continue;
                        }

                        let is_supported = SupportedLanguage::from_path(&path).is_some();

                        if is_supported {
                            match kind {
                                EventKind::Create(_) | EventKind::Modify(_) => {
                                    debug!("Detected change in: {}", path_str);
                                    if let Err(e) = Indexer::update_file(root, &path_str) {
                                        error!("Indexing error: {}", e);
                                    }
                                }
                                EventKind::Remove(_) => {
                                    debug!("Detected removal of: {}", path_str);
                                    if let Err(e) = Indexer::remove_file(root, &path_str) {
                                        error!("Removal error: {}", e);
                                    }
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