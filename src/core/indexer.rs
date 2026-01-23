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

        let ctx_path = Path::new(".amdb");
        let mut db = match ContextDb::open(ctx_path) {
            Ok(db) => db,
            Err(e) => {
                eprintln!("Context DB error: {}", e);
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
                
                match fs::read_to_string(path) {
                    Ok(content) => {
                        match CodeParser::new(lang) {
                            Ok(mut parser) => {
                                match parser.parse(&path_str, &content) {
                                    Ok((symbols, graph, _, warnings)) => {
                                        if let Err(e) = db.save_symbols(&path_str, &symbols) {
                                            eprintln!("  ❌ DB Save Error (Symbols): {}", e);
                                        }
                                        if let Err(e) = db.save_relationships(&path_str, &graph) {
                                            eprintln!("  ❌ DB Save Error (Rels): {}", e);
                                        }
                                        
                                        if !warnings.is_empty() {
                                            if let Err(e) = db.save_warnings(&path_str, &warnings) {
                                                eprintln!("  ❌ DB Save Error (Warnings): {}", e);
                                            }
                                            println!("  🚨 {} Security warning(s) found in {}", warnings.len(), style(&path_str).red());
                                        }
                                        
                                        println!("  + Indexed: {}", style(&path_str).dim());
                                        count += 1;
                                    },
                                    Err(e) => {
                                        eprintln!("  ⚠️ Parse Error in '{}': {}", path_str, e);
                                    }
                                }
                            },
                            Err(e) => {
                                eprintln!("  ⚠️ Parser Init Error for '{}': {}", path_str, e);
                            }
                        }
                    },
                    Err(e) => {
                        eprintln!("  ⚠️ File Read Error '{}': {}", path_str, e);
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
    
    if name == "." { return false; }

    if name == ".amdb" || name == ".git" || name == "target" || name == "node_modules" {
        return true;
    }

    let sensitive_files = [
        ".env", ".env.local", ".env.production",
        "id_rsa", "id_ed25519", ".ssh",
        "secrets.json", "secrets.yaml",
        ".DS_Store"
    ];
    
    if sensitive_files.iter().any(|&f| name.contains(f)) {
        return true;
    }
    if name.ends_with(".pem") || name.ends_with(".key") || name.ends_with(".cert") {
        return true;
    }
    if name.starts_with('.') {
        return true;
    }

    false
}