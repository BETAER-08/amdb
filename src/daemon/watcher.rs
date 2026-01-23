use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::mpsc::channel;
use crate::core::indexer::Indexer;

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
                    if let Some(path) = event.paths.get(0) {
                        let path_str = path.to_string_lossy();
                        
                        if path_str.contains(".database") || path_str.contains(".amdb") {
                            continue;
                        }

                        if path.extension().map_or(false, |ext| ext == "rs" || ext == "py" || ext == "js" || ext == "ts") {
                            println!("Detected change in: {}", path_str);
                            let _ = Indexer::scan_project("."); 
                        }
                    }
                },
                Err(e) => println!("Watch error: {:?}", e),
            }
        }
        Ok(())
    }
}