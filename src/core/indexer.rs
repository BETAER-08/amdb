use walkdir::WalkDir;
use console::style;
use std::path::Path;
use std::fs;
use crate::core::parser::{CodeParser, SupportedLanguage};
use crate::db::ContextDb;
use crate::core::embedding::EmbeddingEngine;
use crate::core::vector_store::VectorStore;
use crate::core::config::AppConfig;

pub struct Indexer;

impl Indexer {
    pub fn scan_project(root: &str) -> anyhow::Result<()> {
        println!("{}", style("🔍 Starting Semantic Indexing (v0.2 Enhanced)...").cyan().bold());

        let config = AppConfig::load();
        
        let embedder = match EmbeddingEngine::new() {
            Ok(e) => Some(e),
            Err(e) => {
                eprintln!("⚠️ Failed to init Embedding Engine: {}", e);
                None
            }
        };

        let ctx_path = Path::new(".amdb");
        if !ctx_path.exists() { fs::create_dir(ctx_path)?; }
        
        let vector_path = ctx_path.join("vector");
        if !vector_path.exists() { fs::create_dir(&vector_path)?; }

        let mut vector_store = VectorStore::load(&vector_path).unwrap_or(VectorStore::new());

        let mut db = match ContextDb::open(ctx_path) {
            Ok(db) => db,
            Err(e) => return Err(anyhow::anyhow!("DB Init failed: {}", e)),
        };

        let walker = WalkDir::new(root).into_iter();
        let mut count = 0;

        for entry in walker.filter_entry(|e| !is_ignored(e, &config.exclude_patterns)) {
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
                        if let Ok((symbols, graph, _, warnings)) = parser.parse(&path_str, &content) {
                            
                            let _ = db.save_symbols(&path_str, &symbols);
                            let _ = db.save_relationships(&path_str, &graph);
                            if !warnings.is_empty() {
                                let _ = db.save_warnings(&path_str, &warnings);
                                println!("  🚨 {} Security warning(s) found in {}", warnings.len(), style(&path_str).red());
                            }

                            if let Some(engine) = &embedder {
                                for symbol in &symbols {
                                    let doc = symbol.docstring.clone().unwrap_or_default();
                                    let semantic_text = format!("Type: {}, Name: {}. Description: {}", 
                                        symbol.kind, symbol.name, doc);
                                    
                                    if let Ok(vec) = engine.embed(&semantic_text) {
                                        let id = format!("{}::{}", path_str, symbol.name);
                                        vector_store.add(id, path_str.clone(), semantic_text, vec);
                                    }
                                }
                            }
                            
                            println!("  + Indexed: {}", style(&path_str).dim());
                            count += 1;
                        }
                    }
                }
            }
        }

        if let Err(e) = vector_store.save(&vector_path) {
            eprintln!("Failed to save vector store: {}", e);
        }

        println!("{}", style(format!("✅ Indexing complete. {} files processed.", count)).green().bold());
        Ok(())
    }
}

fn is_ignored(entry: &walkdir::DirEntry, patterns: &[String]) -> bool {
    let name = entry.file_name().to_string_lossy();
    
    if name == "." { return false; }

    for pattern in patterns {
        if name.contains(pattern) {
            return true;
        }
    }
    
    if name.starts_with('.') && name != "." {
        return true;
    }

    false
}