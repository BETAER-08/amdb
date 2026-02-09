use anyhow::Result;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use rayon::prelude::*;
use ignore::WalkBuilder;
use crate::core::parser::{CodeParser, CodeSymbol, CodeWarning};
use crate::core::vector_store::VectorStore;
use crate::core::embedding::EmbeddingEngine;
use crate::db::ContextDb;
use crate::core::languages::SupportedLanguage;
use crate::core::graph::DependencyGraph;

pub struct Indexer;

struct FileIndexData {
    path: String,
    symbols: Vec<CodeSymbol>,
    graph: DependencyGraph,
    warnings: Vec<CodeWarning>,
    vectors: Vec<(String, String, Vec<f32>)>,
}

impl Indexer {
    pub fn scan_project(root: &str) -> Result<()> {
        let db_dir = Path::new(root).join(".database");
        let vector_path = db_dir.join("vector");

        fs::create_dir_all(&vector_path)?;

        let mut db = ContextDb::open(&db_dir)?;
        let mut vector_store = VectorStore::new();

        let embedder = Arc::new(EmbeddingEngine::new()?);

        if vector_path.exists() {
            if let Ok(store) = VectorStore::load(&vector_path) {
                vector_store = store;
            }
        }

        println!("🔍 Scanning files in {}...", root);

        let walker = WalkBuilder::new(root).build();
        let entries: Vec<_> = walker.filter_map(|e| e.ok())
            .filter(|e| e.file_type().map(|ft| ft.is_file()).unwrap_or(false))
            .collect();

        println!("🚀 Indexing {} files using {} threads...", entries.len(), rayon::current_num_threads());

        let results: Vec<FileIndexData> = entries.par_iter()
            .filter_map(|entry| {
                let path = entry.path();
                let lang = SupportedLanguage::from_path(path)?;
                let mut parser = CodeParser::new(lang).ok()?;
                let code = fs::read_to_string(path).ok()?;
                let path_str = path.to_string_lossy().to_string();

                match parser.parse(&path_str, &code) {
                    Ok((symbols, graph, _, warnings)) => {
                        let mut vectors = Vec::new();

                        for symbol in &symbols {
                            let text = format!(
                                "File: {}\nName: {}\nKind: {}\nDoc: {}",
                                path_str, symbol.name, symbol.kind,
                                symbol.docstring.clone().unwrap_or_default()
                            );

                            if let Ok(embedding) = embedder.embed(&text) {
                                let id = format!("{}::{}", path_str, symbol.name);
                                vectors.push((id, text, embedding));
                            }
                        }

                        Some(FileIndexData {
                            path: path_str,
                            symbols,
                            graph,
                            warnings,
                            vectors,
                        })
                    },
                    Err(_) => None,
                }
            })
            .collect();

        for data in results {
            db.save_symbols(&data.path, &data.symbols)?;
            db.save_relationships(&data.path, &data.graph)?;
            db.save_warnings(&data.path, &data.warnings)?;

            if !data.warnings.is_empty() {
                for warning in &data.warnings {
                    eprintln!(
                        "⚠️  [SECURITY] found in {}:{}: {}",
                        data.path, warning.line, warning.message
                    );
                }
            }

            for (id, text, vector) in data.vectors {
                vector_store.add(data.path.clone(), id, text, vector);
            }
        }

        vector_store.save(&vector_path)?;
        println!("✅ Project indexed successfully at {}", root);
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
}