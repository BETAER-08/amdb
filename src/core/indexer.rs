use crate::core::config::Config;
use crate::core::embedding::EmbeddingEngine;
use crate::core::graph::DependencyGraph;
use crate::core::languages::SupportedLanguage;
use crate::core::parser::{CodeParser, CodeSymbol, CodeWarning};
use crate::core::vector_store::VectorStore;
use crate::db::ContextDb;
use anyhow::Result;
use ignore::WalkBuilder;
use rayon::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::{debug, error, info, warn};

pub struct Indexer;

struct FileIndexData {
    path: String,
    symbols: Vec<CodeSymbol>,
    graph: DependencyGraph,
    warnings: Vec<CodeWarning>,
    vectors: Vec<(String, String, Vec<f32>)>,
}

pub struct IndexWorker {
    db: ContextDb,
    vector_store: VectorStore,
    embedder: EmbeddingEngine,
    vector_path: PathBuf,
}

impl IndexWorker {
    pub fn new(root: &str) -> Result<Self> {
        let config = Config::load(root);
        let db_dir = Path::new(root).join(&config.db_path);
        let vector_path = db_dir.join("vector");

        let db = ContextDb::open(&db_dir)?;
        let vector_store = VectorStore::open(&vector_path)?;
        let embedder = EmbeddingEngine::new()?;

        Ok(Self {
            db,
            vector_store,
            embedder,
            vector_path,
        })
    }

    pub fn update_file(&mut self, path: &str) -> Result<()> {
        use anyhow::Context;

        self.vector_store.remove_by_file(path)?;

        let path_obj = Path::new(path);
        if !path_obj.exists() {
            return Ok(());
        }

        let lang = match SupportedLanguage::from_path(path_obj) {
            Some(l) => l,
            None => return Ok(()),
        };

        let mut parser = CodeParser::new(lang)
            .with_context(|| format!("Failed to create parser for {}", path))?;

        let code = fs::read_to_string(path_obj)
            .with_context(|| format!("Failed to read file {}", path))?;

        let (symbols, graph, _, warnings) = parser
            .parse(path, &code)
            .with_context(|| format!("Failed to parse {}", path))?;

        self.db.save_symbols(path, &symbols)?;
        self.db.save_relationships(path, &graph)?;
        self.db.save_warnings(path, &warnings)?;

        for symbol in &symbols {
            let text = format!(
                "File: {}\nName: {}\nKind: {}\nDoc: {}\nSignature: {}",
                path,
                symbol.name,
                symbol.kind,
                symbol.docstring.clone().unwrap_or_default(),
                symbol.signature.clone().unwrap_or_default()
            );
            match self.embedder.embed(&text) {
                Ok(embedding) => {
                    let id = format!("{}::{}", path, symbol.name);
                    if let Err(e) = self.vector_store.add(path.to_string(), id, text, embedding) {
                        warn!("Failed to add vector for {}: {}", symbol.name, e);
                    }
                }
                Err(e) => warn!("Failed to embed symbol {}: {}", symbol.name, e),
            }
        }

        self.vector_store.save(&self.vector_path)?;
        Ok(())
    }

    pub fn remove_file(&mut self, path: &str) -> Result<()> {
        self.db.remove_file_data(path)?;
        self.vector_store.remove_by_file(path)?;
        self.vector_store.save(&self.vector_path)?;
        Ok(())
    }
}

impl Indexer {
    pub fn scan_project(root: &str) -> Result<()> {
        let config = Config::load(root);
        let db_dir = Path::new(root).join(&config.db_path);
        let vector_path = db_dir.join("vector");

        fs::create_dir_all(&vector_path)?;

        let mut db = ContextDb::open(&db_dir)?;
        let mut vector_store = VectorStore::open(&vector_path)?;

        let embedder = Arc::new(EmbeddingEngine::new()?);
        let excludes = config.ignore_patterns;

        info!("Scanning files in {}...", root);

        let walker = WalkBuilder::new(root)
            .filter_entry(move |entry| {
                let path_str = entry.path().to_string_lossy();
                !excludes.iter().any(|p| path_str.contains(p))
            })
            .build();

        let entries: Vec<_> = walker
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().map(|ft| ft.is_file()).unwrap_or(false))
            .collect();

        info!(
            "Indexing {} files using {} threads...",
            entries.len(),
            rayon::current_num_threads()
        );

        let results: Vec<FileIndexData> = entries
            .par_iter()
            .filter_map(|entry| {
                let path = entry.path();
                let path_str = path.to_string_lossy().to_string();

                let lang = match SupportedLanguage::from_path(path) {
                    Some(l) => l,
                    None => return None,
                };

                let mut parser = match CodeParser::new(lang) {
                    Ok(p) => p,
                    Err(e) => {
                        warn!("Failed to initialize parser for {}: {}", path_str, e);
                        return None;
                    }
                };

                let code = match fs::read_to_string(path) {
                    Ok(c) => c,
                    Err(e) => {
                        warn!("Failed to read file {}: {}", path_str, e);
                        return None;
                    }
                };

                match parser.parse(&path_str, &code) {
                    Ok((symbols, graph, _, warnings)) => {
                        let mut vectors = Vec::new();

                        for symbol in &symbols {
                            let text = format!(
                                "File: {}\nName: {}\nKind: {}\nDoc: {}\nSignature: {}",
                                path_str,
                                symbol.name,
                                symbol.kind,
                                symbol.docstring.clone().unwrap_or_default(),
                                symbol.signature.clone().unwrap_or_default()
                            );

                            match embedder.embed(&text) {
                                Ok(embedding) => {
                                    let id = format!("{}::{}", path_str, symbol.name);
                                    vectors.push((id, text, embedding));
                                }
                                Err(e) => warn!("Failed to embed symbol {} in {}: {}", symbol.name, path_str, e),
                            }
                        }

                        debug!("Parsed: {}", path_str);

                        Some(FileIndexData {
                            path: path_str,
                            symbols,
                            graph,
                            warnings,
                            vectors,
                        })
                    }
                    Err(e) => {
                        warn!("Failed to parse {}: {}", path_str, e);
                        None
                    }
                }
            })
            .collect();

        vector_store.begin_transaction()?;

        for data in results {
            db.save_symbols(&data.path, &data.symbols)?;
            db.save_relationships(&data.path, &data.graph)?;
            db.save_warnings(&data.path, &data.warnings)?;

            if !data.warnings.is_empty() {
                for warning in &data.warnings {
                    warn!(
                        "SECURITY warning in {}:{}: {}",
                        data.path, warning.line, warning.message
                    );
                }
            }

            for (id, text, vector) in data.vectors {
                if let Err(e) = vector_store.add(data.path.clone(), id, text, vector) {
                    error!("Failed to add vector for {}: {}", data.path, e);
                }
            }
        }

        vector_store.commit()?;
        vector_store.save(&vector_path)?;

        info!("Project indexed successfully at {}", root);
        Ok(())
    }

    #[allow(dead_code)]
    pub fn update_file(root: &str, path: &str) -> Result<()> {
        let mut worker = IndexWorker::new(root)?;
        worker.update_file(path)
    }

    #[allow(dead_code)]
    pub fn remove_file(root: &str, path: &str) -> Result<()> {
        let mut worker = IndexWorker::new(root)?;
        worker.remove_file(path)
    }
}