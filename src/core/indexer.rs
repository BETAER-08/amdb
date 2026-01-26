use anyhow::Result;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;
use crate::core::parser::CodeParser;
use crate::core::vector_store::VectorStore;
use crate::core::embedding::EmbeddingEngine;
use crate::db::ContextDb;
use crate::core::languages::SupportedLanguage;

pub struct Indexer;

impl Indexer {
    pub fn scan_project(root: &str) -> Result<()> {
        let db_dir = Path::new(".database");
        if !db_dir.exists() {
            fs::create_dir(db_dir)?;
        }

        let mut db = ContextDb::open(db_dir)?;
        let mut vector_store = VectorStore::new();
        let embedder = EmbeddingEngine::new()?;

        let vector_path = db_dir.join("vector");
        if vector_path.exists() {
            if let Ok(store) = VectorStore::load(&vector_path) {
                vector_store = store;
            }
        }

        for entry in WalkDir::new(root)
            .into_iter()
            .filter_entry(|e| !Indexer::is_ignored(e)) 
        {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                if let Some(lang) = SupportedLanguage::from_path(path) {
                    if let Ok(mut parser) = CodeParser::new(lang) {
                        if let Ok(code) = fs::read_to_string(path) {
                            let path_str = path.to_string_lossy().to_string();
                            
                            if let Ok((symbols, graph, _, warnings)) = parser.parse(&path_str, &code) {
                                db.save_symbols(&path_str, &symbols)?;
                                db.save_relationships(&path_str, &graph)?;
                                db.save_warnings(&path_str, &warnings)?;

                                for symbol in symbols {
                                    let text = format!(
                                        "File: {}\nName: {}\nKind: {}\nDoc: {}", 
                                        path_str, symbol.name, symbol.kind, 
                                        symbol.docstring.clone().unwrap_or_default()
                                    );
                                    
                                    if let Ok(embedding) = embedder.embed(&text) {
                                        let id = format!("{}::{}", path_str, symbol.name);
                                        vector_store.add(path_str.clone(), id, text, embedding);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        vector_store.save(&vector_path)?;
        println!("Project indexed successfully.");
        Ok(())
    }

    fn is_ignored(entry: &walkdir::DirEntry) -> bool {
        let name = entry.file_name().to_string_lossy();
        name.starts_with('.') || 
        name == "target" || 
        name == "node_modules" || 
        name == ".database" || 
        name == ".amdb"
    }
}