use crate::core::config::Config;
use crate::core::embedding::EmbeddingEngine;
use crate::core::graph::DependencyGraph;
use crate::core::languages::SupportedLanguage;
use crate::core::parser::{embedding_text, CodeParser, CodeSymbol, CodeWarning};
use crate::core::symbol::{normalize_path, SymbolRef};
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
    vectors: Vec<(SymbolRef, String, Vec<f32>)>,
}

pub struct IndexWorker {
    root: String,
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
        if db.needs_reindex {
            warn!("Legacy schema detected; run 'amdb init' again for a full rebuild of relationship data.");
        }
        let vector_store = VectorStore::open(&vector_path)?;
        let embedder = EmbeddingEngine::new()?;

        Ok(Self {
            root: root.to_string(),
            db,
            vector_store,
            embedder,
            vector_path,
        })
    }

    pub fn update_file(&mut self, path: &str) -> Result<()> {
        use anyhow::Context;

        let path_obj = Path::new(path);
        let stored_path = normalize_path(&self.root, path_obj);

        self.vector_store.remove_by_file(&stored_path)?;

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
            .parse(&stored_path, &code)
            .with_context(|| format!("Failed to parse {}", path))?;

        self.db.save_symbols(&stored_path, &symbols)?;
        self.db.save_relationships(&stored_path, &graph)?;
        self.db.save_warnings(&stored_path, &warnings)?;

        for symbol in &symbols {
            let text = embedding_text(symbol);
            match self.embedder.embed(&text) {
                Ok(embedding) => {
                    let sym = SymbolRef::new(stored_path.clone(), symbol.name.clone());
                    if let Err(e) = self.vector_store.add(&sym, text, embedding) {
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
        let stored_path = normalize_path(&self.root, Path::new(path));
        self.db.remove_file_data(&stored_path)?;
        self.vector_store.remove_by_file(&stored_path)?;
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
        if db.needs_reindex {
            info!("Legacy schema detected; rebuilding relationship data from this scan.");
        }
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
                let stored_path = normalize_path(root, path);

                let lang = match SupportedLanguage::from_path(path) {
                    Some(l) => l,
                    None => return None,
                };

                let mut parser = match CodeParser::new(lang) {
                    Ok(p) => p,
                    Err(e) => {
                        warn!("Failed to initialize parser for {}: {}", stored_path, e);
                        return None;
                    }
                };

                let code = match fs::read_to_string(path) {
                    Ok(c) => c,
                    Err(e) => {
                        warn!("Failed to read file {}: {}", stored_path, e);
                        return None;
                    }
                };

                match parser.parse(&stored_path, &code) {
                    Ok((symbols, graph, _, warnings)) => {
                        let mut vectors = Vec::new();

                        for symbol in &symbols {
                            let text = embedding_text(symbol);

                            match embedder.embed(&text) {
                                Ok(embedding) => {
                                    let sym = SymbolRef::new(stored_path.clone(), symbol.name.clone());
                                    vectors.push((sym, text, embedding));
                                }
                                Err(e) => warn!("Failed to embed symbol {} in {}: {}", symbol.name, stored_path, e),
                            }
                        }

                        debug!("Parsed: {}", stored_path);

                        Some(FileIndexData {
                            path: stored_path,
                            symbols,
                            graph,
                            warnings,
                            vectors,
                        })
                    }
                    Err(e) => {
                        warn!("Failed to parse {}: {}", stored_path, e);
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

            for (sym, text, vector) in data.vectors {
                if let Err(e) = vector_store.add(&sym, text, vector) {
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
