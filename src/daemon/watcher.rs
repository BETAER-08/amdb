use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher, EventKind};
use notify::event::{ModifyKind, RenameMode};
use std::path::Path;
use std::sync::mpsc::channel;
use crate::core::indexer::Indexer;
use crate::core::languages::SupportedLanguage;

pub struct FileWatcher;

impl FileWatcher {
    pub async fn watch(root: &str) -> anyhow::Result<()> {
        let (tx, rx) = channel();
        let mut watcher = RecommendedWatcher::new(tx, Config::default())?;

        watcher.watch(Path::new(root), RecursiveMode::Recursive)?;

        println!("Watcher started on: {}", root);

        for res in rx {
            match res {
                Ok(event) => {
                    let kind = event.kind;

                    if let EventKind::Modify(ModifyKind::Name(RenameMode::Both)) = kind {
                        if event.paths.len() == 2 {
                            let old_path = event.paths[0].to_string_lossy();
                            let new_path = event.paths[1].to_string_lossy();

                            println!("Rename detected: {} -> {}", old_path, new_path);

                            if !old_path.contains(".database") && !old_path.contains(".amdb") {
                                if let Err(e) = Indexer::remove_file(root, &old_path) {
                                    eprintln!("Remove error: {}", e);
                                }
                            }

                            if !new_path.contains(".database") && !new_path.contains(".amdb") {
                                if let Err(e) = Indexer::update_file(root, &new_path) {
                                    eprintln!("Update error: {}", e);
                                }
                            }
                            continue;
                        }
                    }

                    for path in event.paths {
                        let path_str = path.to_string_lossy().to_string();

                        if path_str.contains(".database") || path_str.contains(".amdb") {
                            continue;
                        }

                        let is_supported = SupportedLanguage::from_path(&path).is_some();

                        if is_supported {
                            match kind {
                                EventKind::Create(_) | EventKind::Modify(_) => {
                                    println!("Detected change in: {}", path_str);
                                    if let Err(e) = Indexer::update_file(root, &path_str) {
                                        eprintln!("Indexing error: {}", e);
                                    }
                                },
                                EventKind::Remove(_) => {
                                    println!("Detected removal of: {}", path_str);
                                    if let Err(e) = Indexer::remove_file(root, &path_str) {
                                        eprintln!("Removal error: {}", e);
                                    }
                                },
                                _ => {}
                            }
                        }
                    }
                },
                Err(e) => println!("Watch error: {:?}", e),
            }
        }
        Ok(())
    }
}