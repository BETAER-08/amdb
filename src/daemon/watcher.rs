use anyhow::Result;
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::mpsc::channel;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use crate::core::parser::{CodeParser, SupportedLanguage};
use crate::db::ContextDb;
use console::style;

#[allow(dead_code)]
pub struct SystemWatcher;

impl SystemWatcher {
    pub fn start(path: &str) -> Result<()> {
        let (tx, rx) = channel();
        
        let mut watcher = RecommendedWatcher::new(tx, Config::default())?;
        
        watcher.watch(Path::new(path), RecursiveMode::Recursive)?;

        println!("{}", style(format!("Watching '{}'...", path)).bold().green());

        let mut last_processed: HashMap<String, Instant> = HashMap::new();

        for res in rx {
            match res {
                Ok(event) => {
                    for path in event.paths {
                        let path_str = path.to_string_lossy().to_string();

                        if path_str.contains("/target/") || path_str.contains("/.git/") || path_str.contains(".amdb") {
                            continue;
                        }

                        if !Path::new(&path_str).exists() {
                            continue;
                        }

                        if SupportedLanguage::from_path(Path::new(&path_str)).is_none() {
                            continue;
                        }

                        let now = Instant::now();
                        if let Some(last_time) = last_processed.get(&path_str) {
                            if now.duration_since(*last_time) < Duration::from_millis(500) {
                                continue;
                            }
                        }

                        if let Err(e) = Self::process_file(&path_str) {
                            eprintln!("Error processing file: {}", e);
                        } else {
                            last_processed.insert(path_str, now);
                        }
                    }
                },
                Err(e) => println!("Watch error: {:?}", e),
            }
        }

        Ok(())
    }

    fn process_file(file_path: &str) -> Result<()> {
        let path = Path::new(file_path);
        
        let content = std::fs::read_to_string(path)?;
        let lang = SupportedLanguage::from_path(path).unwrap();
        
        let mut parser = CodeParser::new(lang)?;
        
        let (symbols, graph, _) = parser.parse(file_path, &content)?;

        let ctx_path = Path::new(".amdb");
        if !ctx_path.exists() { return Ok(()); }

        let mut db = ContextDb::open(ctx_path)?;
        
        db.save_symbols(file_path, &symbols)?;
        db.save_relationships(file_path, &graph)?;

        println!("{}", style(format!("Synced: {}", file_path)).green());
        Ok(())
    }
}