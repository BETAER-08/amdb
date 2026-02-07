use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher, EventKind};
use notify::event::{ModifyKind, RenameMode};
use std::path::Path;
use std::sync::mpsc::channel;
use crate::core::indexer::Indexer;
use crate::core::languages::SupportedLanguage;

pub struct FileWatcher;

impl FileWatcher {
    pub async fn watch(path: &str) -> anyhow::Result<()> {
        let (tx, rx) = channel();
        let mut watcher = RecommendedWatcher::new(tx, Config::default())?;

        watcher.watch(Path::new(path), RecursiveMode::Recursive)?;

        println!("Watcher started on: {}", path);

        for res in rx {
            match res {
                Ok(event) => {
                    let kind = event.kind;

                    // Rename 이벤트 처리 (notify 6.0 기준)
                    if let EventKind::Modify(ModifyKind::Name(RenameMode::Both)) = kind {
                        if event.paths.len() == 2 {
                            let old_path = event.paths[0].to_string_lossy();
                            let new_path = event.paths[1].to_string_lossy();

                            println!("Rename detected: {} -> {}", old_path, new_path);

                            // 무시할 경로인지 체크 (간단히 문자열 포함 여부 등)
                            if !old_path.contains(".database") && !old_path.contains(".amdb") {
                                if let Err(e) = Indexer::remove_file(&old_path) {
                                    eprintln!("Remove error: {}", e);
                                }
                            }

                            if !new_path.contains(".database") && !new_path.contains(".amdb") {
                                if let Err(e) = Indexer::update_file(&new_path) {
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

                        // 언어 지원 파일인지 확인 (삭제 시에는 파일이 없으므로 확장자로 추측하거나, Indexer 내부에서 처리 권장)
                        // 단, 여기서는 Path 객체가 살아있으므로 확장자 체크 가능
                        let is_supported = SupportedLanguage::from_path(&path).is_some();

                        if is_supported {
                            match kind {
                                EventKind::Create(_) | EventKind::Modify(_) => {
                                    println!("Detected change in: {}", path_str);
                                    if let Err(e) = Indexer::update_file(&path_str) {
                                        eprintln!("Indexing error: {}", e);
                                    }
                                },
                                EventKind::Remove(_) => {
                                    println!("Detected removal of: {}", path_str);
                                    if let Err(e) = Indexer::remove_file(&path_str) {
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