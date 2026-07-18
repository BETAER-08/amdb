use crate::core::config::Config;
use crate::core::embedding::EmbeddingEngine;
use crate::core::graph::DependencyGraph;
use crate::core::languages::SupportedLanguage;
use crate::core::parser::{embedding_text, CodeParser, CodeSymbol, CodeWarning};
use crate::core::symbol::{normalize_path, SymbolRef, SymbolResolver};
use crate::core::vector_store::VectorStore;
use crate::db::ContextDb;
use anyhow::Result;
use ignore::WalkBuilder;
use rayon::prelude::*;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::{debug, error, info, warn};
use xxhash_rust::xxh3::xxh3_64;

pub struct Indexer;

pub fn content_hash(bytes: &[u8]) -> String {
    format!("{:016x}", xxh3_64(bytes))
}

pub fn reresolve_delta(
    db: &mut ContextDb,
    delta_files: &[String],
    affected_names: &HashSet<String>,
) -> Result<()> {
    let mut collected = Vec::new();
    for file in delta_files {
        collected.extend(db.get_relationships(file)?);
    }
    for name in affected_names {
        collected.extend(db.get_relationships_by_callee(name)?);
    }

    let mut seen = HashSet::new();
    let edges: Vec<_> = collected
        .into_iter()
        .filter(|r| {
            seen.insert((
                r.caller.file.clone(),
                r.caller.name.clone(),
                r.callee.clone(),
            ))
        })
        .collect();

    let callee_names: HashSet<&str> = edges.iter().map(|r| r.callee.as_str()).collect();
    let mut resolver = SymbolResolver::new();
    for name in callee_names {
        for file in db.get_defining_files(name)? {
            resolver.register(&file, name);
        }
    }

    let resolved: Vec<(String, String, String, Option<String>)> = edges
        .iter()
        .map(|r| {
            let callee_file = resolver.resolve(&r.caller.file, &r.callee);
            (
                r.caller.file.clone(),
                r.caller.name.clone(),
                r.callee.clone(),
                callee_file,
            )
        })
        .collect();

    db.set_callee_files(&resolved)?;
    Ok(())
}

struct ScannedFile {
    stored_path: String,
    path: PathBuf,
    hash: Option<String>,
}

struct FileIndexData {
    path: String,
    hash: String,
    symbols: Vec<CodeSymbol>,
    graph: DependencyGraph,
    warnings: Vec<CodeWarning>,
    vectors: Vec<(SymbolRef, String, Vec<f32>)>,
}

fn parse_and_embed(
    stored_path: &str,
    path: &Path,
    hash: &str,
    embedder: &EmbeddingEngine,
) -> Option<FileIndexData> {
    let lang = SupportedLanguage::from_path(path)?;

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

    match parser.parse(stored_path, &code) {
        Ok((symbols, graph, _, warnings)) => {
            let mut vectors = Vec::new();

            for symbol in &symbols {
                let text = embedding_text(symbol);

                match embedder.embed(&text) {
                    Ok(embedding) => {
                        let sym = SymbolRef::new(stored_path.to_string(), symbol.name.clone());
                        vectors.push((sym, text, embedding));
                    }
                    Err(e) => warn!(
                        "Failed to embed symbol {} in {}: {}",
                        symbol.name, stored_path, e
                    ),
                }
            }

            debug!("Parsed: {}", stored_path);

            Some(FileIndexData {
                path: stored_path.to_string(),
                hash: hash.to_string(),
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
        let db_dir = config.db_dir(root);
        let vector_path = config.vector_dir(root);

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

        if !path_obj.exists() {
            return self.remove_file(path);
        }

        let stored_path = normalize_path(&self.root, path_obj);

        let lang = match SupportedLanguage::from_path(path_obj) {
            Some(l) => l,
            None => return Ok(()),
        };

        let bytes = fs::read(path_obj).with_context(|| format!("Failed to read file {}", path))?;
        let hash = content_hash(&bytes);

        if self.db.get_file_hash(&stored_path)?.as_deref() == Some(hash.as_str()) {
            debug!("Unchanged, skipping: {}", stored_path);
            return Ok(());
        }

        let code = String::from_utf8(bytes)
            .with_context(|| format!("File is not valid UTF-8: {}", path))?;

        let mut parser = CodeParser::new(lang)
            .with_context(|| format!("Failed to create parser for {}", path))?;

        let (symbols, graph, _, warnings) = parser
            .parse(&stored_path, &code)
            .with_context(|| format!("Failed to parse {}", path))?;

        let mut affected_names: HashSet<String> = self
            .db
            .get_symbols(&stored_path)?
            .into_iter()
            .map(|s| s.name)
            .collect();
        for symbol in &symbols {
            affected_names.insert(symbol.name.clone());
        }

        self.db.save_symbols(&stored_path, &symbols)?;
        self.db.save_relationships(&stored_path, &graph)?;
        self.db.save_warnings(&stored_path, &warnings)?;
        self.db.upsert_file_hash(&stored_path, &hash)?;

        reresolve_delta(
            &mut self.db,
            std::slice::from_ref(&stored_path),
            &affected_names,
        )?;

        self.vector_store.remove_by_file(&stored_path)?;

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

        let affected_names: HashSet<String> = self
            .db
            .get_symbols(&stored_path)?
            .into_iter()
            .map(|s| s.name)
            .collect();

        self.db.remove_file_data(&stored_path)?;
        self.vector_store.remove_by_file(&stored_path)?;
        self.vector_store.save(&self.vector_path)?;

        reresolve_delta(&mut self.db, &[], &affected_names)?;
        Ok(())
    }
}

impl Indexer {
    pub fn scan_project(root: &str) -> Result<()> {
        let config = Config::load(root);
        let db_dir = config.db_dir(root);
        let vector_path = config.vector_dir(root);

        fs::create_dir_all(&vector_path)?;

        let mut db = ContextDb::open(&db_dir)?;
        if db.needs_reindex {
            info!("Legacy schema detected; rebuilding relationship data from this scan.");
        }
        let mut vector_store = VectorStore::open(&vector_path)?;

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

        let scanned: Vec<ScannedFile> = entries
            .par_iter()
            .filter_map(|entry| {
                let path = entry.path();
                SupportedLanguage::from_path(path)?;
                let stored_path = normalize_path(root, path);

                let hash = match fs::read(path) {
                    Ok(bytes) => Some(content_hash(&bytes)),
                    Err(e) => {
                        warn!("Failed to read file {}: {}", stored_path, e);
                        None
                    }
                };

                Some(ScannedFile {
                    stored_path,
                    path: path.to_path_buf(),
                    hash,
                })
            })
            .collect();

        let stored_hashes = db.get_file_hashes()?;
        let present: HashSet<&str> = scanned.iter().map(|s| s.stored_path.as_str()).collect();
        let removed: Vec<String> = stored_hashes
            .keys()
            .filter(|k| !present.contains(k.as_str()))
            .cloned()
            .collect();

        let mut unchanged_count = 0usize;
        let mut changed_count = 0usize;
        let mut added_count = 0usize;
        let mut to_index: Vec<(String, PathBuf, String)> = Vec::new();

        for file in scanned {
            let Some(hash) = file.hash else { continue };
            match stored_hashes.get(&file.stored_path) {
                Some(prev) if *prev == hash => unchanged_count += 1,
                Some(_) => {
                    changed_count += 1;
                    to_index.push((file.stored_path, file.path, hash));
                }
                None => {
                    added_count += 1;
                    to_index.push((file.stored_path, file.path, hash));
                }
            }
        }

        info!(
            "Files: {} unchanged, {} changed, {} added, {} removed",
            unchanged_count,
            changed_count,
            added_count,
            removed.len()
        );

        let mut affected_names: HashSet<String> = HashSet::new();
        for (stored_path, _, _) in &to_index {
            for sym in db.get_symbols(stored_path)? {
                affected_names.insert(sym.name);
            }
        }
        for path in &removed {
            for sym in db.get_symbols(path)? {
                affected_names.insert(sym.name);
            }
            db.remove_file_data(path)?;
            vector_store.remove_by_file(path)?;
        }

        info!(
            "Indexing {} files using {} threads...",
            to_index.len(),
            rayon::current_num_threads()
        );

        let embed_calls;
        let results: Vec<FileIndexData> = if to_index.is_empty() {
            embed_calls = 0;
            Vec::new()
        } else {
            let embedder = Arc::new(EmbeddingEngine::new()?);
            let results: Vec<FileIndexData> = to_index
                .par_iter()
                .filter_map(|(stored_path, path, hash)| {
                    parse_and_embed(stored_path, path, hash, &embedder)
                })
                .collect();
            embed_calls = embedder.call_count();
            results
        };

        info!("Embedding calls: {}", embed_calls);

        for data in &results {
            db.save_symbols(&data.path, &data.symbols)?;
            db.save_relationships(&data.path, &data.graph)?;
            db.save_warnings(&data.path, &data.warnings)?;
            db.upsert_file_hash(&data.path, &data.hash)?;

            for sym in &data.symbols {
                affected_names.insert(sym.name.clone());
            }

            if !data.warnings.is_empty() {
                for warning in &data.warnings {
                    warn!(
                        "SECURITY warning in {}:{}: {}",
                        data.path, warning.line, warning.message
                    );
                }
            }
        }

        let delta_files: Vec<String> = results.iter().map(|d| d.path.clone()).collect();
        reresolve_delta(&mut db, &delta_files, &affected_names)?;

        vector_store.begin_transaction()?;

        for data in results {
            if let Err(e) = vector_store.remove_by_file(&data.path) {
                error!("Failed to remove old vectors for {}: {}", data.path, e);
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_symbol(name: &str) -> CodeSymbol {
        CodeSymbol {
            kind: "function".to_string(),
            name: name.to_string(),
            line: 1,
            docstring: None,
            signature: None,
            is_public: true,
        }
    }

    fn edge_graph(file: &str, caller: &str, callee: &str) -> DependencyGraph {
        let mut graph = DependencyGraph::new();
        graph.add_edge(file, caller, callee);
        graph
    }

    fn callee_file_of(db: &ContextDb, file: &str) -> Option<String> {
        let rels = db.get_relationships(file).unwrap();
        assert_eq!(rels.len(), 1);
        rels[0].callee_file.clone()
    }

    fn relationships_snapshot(db_path: &Path) -> Vec<(String, String, String, Option<String>)> {
        let conn = rusqlite::Connection::open(db_path.join("context.db")).unwrap();
        let mut stmt = conn
            .prepare("SELECT file_path, caller, callee, callee_file FROM relationships ORDER BY file_path, caller, callee")
            .unwrap();
        let rows = stmt
            .query_map([], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
            })
            .unwrap();
        rows.map(|r| r.unwrap()).collect()
    }

    #[test]
    fn test_reresolve_prefers_same_file_definition() {
        let dir = TempDir::new().unwrap();
        let mut db = ContextDb::open(dir.path()).unwrap();

        db.save_symbols(
            "caller.rs",
            &[make_symbol("entry_fn"), make_symbol("local_fn")],
        )
        .unwrap();
        db.save_symbols("other.rs", &[make_symbol("local_fn")])
            .unwrap();
        db.save_relationships(
            "caller.rs",
            &edge_graph("caller.rs", "entry_fn", "local_fn"),
        )
        .unwrap();

        reresolve_delta(&mut db, &["caller.rs".to_string()], &HashSet::new()).unwrap();

        assert_eq!(
            callee_file_of(&db, "caller.rs").as_deref(),
            Some("caller.rs")
        );
    }

    #[test]
    fn test_reresolve_applies_global_unique() {
        let dir = TempDir::new().unwrap();
        let mut db = ContextDb::open(dir.path()).unwrap();

        db.save_symbols("definer.rs", &[make_symbol("target_fn")])
            .unwrap();
        db.save_symbols("caller.rs", &[make_symbol("entry_fn")])
            .unwrap();
        db.save_relationships(
            "caller.rs",
            &edge_graph("caller.rs", "entry_fn", "target_fn"),
        )
        .unwrap();

        reresolve_delta(&mut db, &["caller.rs".to_string()], &HashSet::new()).unwrap();

        assert_eq!(
            callee_file_of(&db, "caller.rs").as_deref(),
            Some("definer.rs")
        );
    }

    #[test]
    fn test_reresolve_affected_name_invalidates_stale_attribution() {
        let dir = TempDir::new().unwrap();
        let mut db = ContextDb::open(dir.path()).unwrap();

        db.save_symbols("definer.rs", &[make_symbol("target_fn")])
            .unwrap();
        db.save_symbols("caller.rs", &[make_symbol("entry_fn")])
            .unwrap();
        db.save_relationships(
            "caller.rs",
            &edge_graph("caller.rs", "entry_fn", "target_fn"),
        )
        .unwrap();
        reresolve_delta(&mut db, &["caller.rs".to_string()], &HashSet::new()).unwrap();
        assert_eq!(
            callee_file_of(&db, "caller.rs").as_deref(),
            Some("definer.rs")
        );

        db.save_symbols("second.rs", &[make_symbol("target_fn")])
            .unwrap();
        let affected: HashSet<String> = ["target_fn".to_string()].into_iter().collect();
        reresolve_delta(&mut db, &["second.rs".to_string()], &affected).unwrap();

        assert_eq!(callee_file_of(&db, "caller.rs"), None);

        db.remove_file_data("second.rs").unwrap();
        reresolve_delta(&mut db, &[], &affected).unwrap();

        assert_eq!(
            callee_file_of(&db, "caller.rs").as_deref(),
            Some("definer.rs")
        );
    }

    #[test]
    fn test_daemon_reresolves_on_duplicate_introduced_and_removed() {
        let root_dir = TempDir::new().unwrap();
        let root = root_dir.path();

        fs::write(root.join("definer.rs"), "fn unique_fn() {}").unwrap();
        fs::write(root.join("caller.rs"), "fn entry_fn() { unique_fn(); }").unwrap();

        Indexer::scan_project(root.to_str().unwrap()).unwrap();

        let db_path = root.join(".database");
        {
            let db = ContextDb::open(&db_path).unwrap();
            assert_eq!(
                callee_file_of(&db, "caller.rs").as_deref(),
                Some("definer.rs")
            );
        }

        let mut worker = IndexWorker::new(root.to_str().unwrap()).unwrap();

        let duplicate = root.join("duplicate.rs");
        fs::write(&duplicate, "fn unique_fn() {}").unwrap();
        worker.update_file(duplicate.to_str().unwrap()).unwrap();

        {
            let db = ContextDb::open(&db_path).unwrap();
            assert_eq!(callee_file_of(&db, "caller.rs"), None);
        }

        fs::remove_file(&duplicate).unwrap();
        worker.update_file(duplicate.to_str().unwrap()).unwrap();

        {
            let db = ContextDb::open(&db_path).unwrap();
            assert_eq!(
                callee_file_of(&db, "caller.rs").as_deref(),
                Some("definer.rs")
            );
        }
    }

    #[test]
    fn test_daemon_and_init_agree() {
        let files = [
            ("f1.rs", "fn alpha() { beta(); gamma(); }\nfn beta() {}"),
            ("f2.rs", "fn gamma() { beta(); }"),
            ("f3.rs", "fn beta() {}"),
        ];

        let daemon_dir = TempDir::new().unwrap();
        let daemon_root = daemon_dir.path();
        Indexer::scan_project(daemon_root.to_str().unwrap()).unwrap();

        let mut worker = IndexWorker::new(daemon_root.to_str().unwrap()).unwrap();
        for (name, code) in files {
            let path = daemon_root.join(name);
            fs::write(&path, code).unwrap();
            worker.update_file(path.to_str().unwrap()).unwrap();
        }

        let init_dir = TempDir::new().unwrap();
        let init_root = init_dir.path();
        for (name, code) in files {
            fs::write(init_root.join(name), code).unwrap();
        }
        Indexer::scan_project(init_root.to_str().unwrap()).unwrap();

        let daemon_rels = relationships_snapshot(&daemon_root.join(".database"));
        let init_rels = relationships_snapshot(&init_root.join(".database"));

        assert!(!init_rels.is_empty());
        assert_eq!(daemon_rels, init_rels);
    }
}
