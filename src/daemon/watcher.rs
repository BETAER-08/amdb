use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
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
                    for path in event.paths {
                        let path_str = path.to_string_lossy();

                        if path_str.contains(".database") || path_str.contains(".amdb") {
                            continue;
                        }

                        if path.is_file() && SupportedLanguage::from_path(&path).is_some() {
                            println!("Detected change in: {}", path_str);
                            if let Err(e) = Indexer::scan_project(".") {
                                eprintln!("Indexing error: {}", e);
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