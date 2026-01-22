use walkdir::WalkDir;
use console::style;
use std::path::Path;
use std::fs;
use crate::core::parser::{CodeParser, SupportedLanguage};
use crate::db::ContextDb;

pub struct Indexer;

impl Indexer {
    pub fn scan_project(root: &str) -> anyhow::Result<()> {
        println!("{}", style("🔍 Starting initial project scan...").cyan().bold());

        let amdb_path = Path::new(".amdb");
        let mut db = match ContextDb::open(amdb_path) {
            Ok(db) => db,
            Err(_) => {
                eprintln!("Context DB not found. Skipping index.");
                return Ok(());
            }
        };

        let walker = WalkDir::new(root).into_iter();
        let mut count = 0;

        for entry in walker.filter_entry(|e| !is_ignored(e)) {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };
            
            let path = entry.path();
            if !path.is_file() { continue; }
            if let Some(lang) = SupportedLanguage::from_path(path) {
                let path_str = path.to_string_lossy().to_string();
                
                if let Ok(content) = fs::read_to_string(path) {
                   if let Ok(mut parser) = CodeParser::new(lang) {
                       if let Ok((symbols, graph)) = parser.parse(&path_str, &content) {
                           let _ = db.save_symbols(&path_str, &symbols);
                           let _ = db.save_relationships(&path_str, &graph);
                           
                           println!("  + Indexed: {}", style(&path_str).dim());
                           count += 1;
                       }
                   }
                }
            }
        }

        println!("{}", style(format!("✅ Indexing complete. {} files learned.", count)).green().bold());
        Ok(())
    }
}

fn is_ignored(entry: &walkdir::DirEntry) -> bool {
    let name = entry.file_name().to_string_lossy();
    name == ".amdb" || name == ".git" || name == "target" || name == "node_modules" || name.starts_with('.')
}