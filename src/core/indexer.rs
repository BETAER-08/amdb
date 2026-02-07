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
        let db_dir = Path::new(root).join(".database");
        let vector_path = db_dir.join("vector");

        fs::create_dir_all(&vector_path)?;

        let mut db = ContextDb::open(&db_dir)?;
        let mut vector_store = VectorStore::new();
        let embedder = EmbeddingEngine::new()?;

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
                    match CodeParser::new(lang) {
                        Ok(mut parser) => {
                            if let Ok(code) = fs::read_to_string(path) {
                                let path_str = path.to_string_lossy().to_string();

                                match parser.parse(&path_str, &code) {
                                    Ok((symbols, graph, _, warnings)) => {
                                        println!("Indexed: {}", path_str);
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
                                    },
                                    Err(e) => println!("Parse Error [{}]: {:?}", path_str, e),
                                }
                            }
                        },
                        Err(e) => println!("Parser Init Error [{}]: {:?}", path.display(), e),
                    }
                }
            }
        }

        vector_store.save(&vector_path)?;
        println!("Project indexed successfully at {}", root);
        Ok(())
    }

    pub fn update_file(root: &str, path: &str) -> Result<()> {
        let db_dir = Path::new(root).join(".database");
        let vector_path = db_dir.join("vector");

        let mut db = ContextDb::open(&db_dir)?;
        let mut vector_store = VectorStore::load(&vector_path).unwrap_or_else(|_| VectorStore::new());
        let embedder = EmbeddingEngine::new()?;

        vector_store.remove_by_file(path);

        let path_obj = Path::new(path);
        if path_obj.exists() {
            if let Some(lang) = SupportedLanguage::from_path(path_obj) {
                if let Ok(mut parser) = CodeParser::new(lang) {
                    if let Ok(code) = fs::read_to_string(path_obj) {
                        if let Ok((symbols, graph, _, warnings)) = parser.parse(path, &code) {
                            db.save_symbols(path, &symbols)?;
                            db.save_relationships(path, &graph)?;
                            db.save_warnings(path, &warnings)?;

                            for symbol in symbols {
                                let text = format!(
                                    "File: {}\nName: {}\nKind: {}\nDoc: {}",
                                    path, symbol.name, symbol.kind,
                                    symbol.docstring.clone().unwrap_or_default()
                                );

                                if let Ok(embedding) = embedder.embed(&text) {
                                    let id = format!("{}::{}", path, symbol.name);
                                    vector_store.add(path.to_string(), id, text, embedding);
                                }
                            }
                        }
                    }
                }
            }
        }

        vector_store.save(&vector_path)?;
        println!("Updated: {}", path);
        Ok(())
    }

    pub fn remove_file(root: &str, path: &str) -> Result<()> {
        let db_dir = Path::new(root).join(".database");
        let vector_path = db_dir.join("vector");

        let mut db = ContextDb::open(&db_dir)?;
        let mut vector_store = VectorStore::load(&vector_path).unwrap_or_else(|_| VectorStore::new());

        db.remove_file_data(path)?;
        vector_store.remove_by_file(path);

        vector_store.save(&vector_path)?;
        println!("Removed: {}", path);
        Ok(())
    }

fn is_ignored(entry: &walkdir::DirEntry) -> bool {
        let name = entry.file_name().to_string_lossy();
        if name == "." { return false; }
        name.starts_with('.') ||
            name == "target" ||
            name == "node_modules" ||
            name == ".database" ||
            name == ".amdb"
    }
}