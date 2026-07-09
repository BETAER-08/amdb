use crate::core::config::Config;
use crate::core::embedding::EmbeddingEngine;
use crate::core::graph::DependencyGraph;
use crate::core::symbol::SymbolRef;
use crate::core::vector_store::VectorStore;
use crate::db::{ContextDb, Relationship};
use anyhow::Result;
use console::style;
use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

pub struct ContextGenerator;

pub struct GraphData {
    pub all_edges: Vec<Relationship>,
    pub all_files: Vec<String>,
    pub graph: DependencyGraph,
    pub symbol_to_files: HashMap<String, Vec<String>>,
}

impl ContextGenerator {
    pub async fn generate(focus_query: Option<String>, depth: u8) -> Result<()> {
        let config = Config::load(".");
        let db_dir = config.db_dir(".");
        let vector_path = config.vector_dir(".");
        let output_dir = Path::new(".amdb");

        if !output_dir.exists() {
            fs::create_dir(output_dir)?;
        }

        if !db_dir.exists() {
            anyhow::bail!("Database not found. Run 'amdb init' first.");
        }

        let db = ContextDb::open(&db_dir)?;
        let data = Self::load_graph_data(&db)?;

        let target_files: Vec<String>;
        let output_filename: String;

        if let Some(query) = &focus_query {
            output_filename = Self::focus_filename(query);

            println!(
                "{}",
                style(format!(
                    "Filtering context for: '{}' with depth {}...",
                    query, depth
                ))
                .cyan()
            );

            match Self::compute_focus_files(&db, &vector_path, &data, query, depth, 10)? {
                Some(files) => target_files = files,
                None => {
                    println!(
                        "{}",
                        style("No matches found. Falling back to full context.").yellow()
                    );
                    target_files = data.all_files.clone();
                }
            }
        } else {
            output_filename = "context.md".to_string();
            println!("{}", style("Generating full project context...").blue());
            target_files = data.all_files.clone();
        }

        let content = Self::render_report(
            &db,
            &data,
            &target_files,
            focus_query.as_deref(),
            &output_filename,
        )?;

        let output_path = output_dir.join(&output_filename);
        let mut file = File::create(&output_path)?;
        file.write_all(content.as_bytes())?;

        println!(
            "{}",
            style(format!("Generated: {}", output_path.display()))
                .green()
                .bold()
        );
        Ok(())
    }

    pub fn focus_filename(query: &str) -> String {
        let safe_name = query.replace(" ", "-").replace("/", "-").to_lowercase();
        format!("{}.md", safe_name)
    }

    pub fn load_graph_data(db: &ContextDb) -> Result<GraphData> {
        let all_edges = db.get_all_relationships()?;
        let all_files = db.get_all_files()?;

        let mut graph = DependencyGraph::new();
        for edge in &all_edges {
            graph
                .edges
                .entry(edge.caller.clone())
                .or_default()
                .insert(edge.callee.clone());
            graph
                .reverse_edges
                .entry(edge.callee.clone())
                .or_default()
                .insert(edge.caller.clone());
        }

        let mut symbol_to_files: HashMap<String, Vec<String>> = HashMap::new();

        for file in &all_files {
            if let Ok(symbols) = db.get_symbols(file) {
                for sym in symbols {
                    symbol_to_files
                        .entry(sym.name)
                        .or_default()
                        .push(file.clone());
                }
            }
        }

        Ok(GraphData {
            all_edges,
            all_files,
            graph,
            symbol_to_files,
        })
    }

    pub fn compute_focus_files(
        db: &ContextDb,
        vector_path: &Path,
        data: &GraphData,
        query: &str,
        depth: u8,
        limit: usize,
    ) -> Result<Option<Vec<String>>> {
        let embedder = EmbeddingEngine::new()?;
        let paths = Self::resolve_focus_targets(
            vector_path,
            &data.all_files,
            db,
            query,
            &embedder,
            &data.graph,
            limit,
        )?;

        if paths.is_empty() {
            return Ok(None);
        }

        let mut file_graph: HashMap<String, HashSet<String>> = HashMap::new();
        for edge in &data.all_edges {
            let caller_file = &edge.caller.file;
            if let Some(callee_files) = data.symbol_to_files.get(&edge.callee) {
                for callee_file in callee_files {
                    if caller_file != callee_file {
                        file_graph
                            .entry(caller_file.clone())
                            .or_default()
                            .insert(callee_file.clone());
                    }
                }
            }
        }

        Ok(Some(Self::expand_graph_depth(paths, &file_graph, depth)))
    }

    pub fn render_report(
        db: &ContextDb,
        data: &GraphData,
        target_files: &[String],
        focus_query: Option<&str>,
        output_filename: &str,
    ) -> Result<String> {
        let mapped_edges: Vec<(&SymbolRef, &str, Option<&str>)> = data
            .all_edges
            .iter()
            .map(|e| (&e.caller, e.callee.as_str(), e.callee_file.as_deref()))
            .collect();

        Self::format_markdown_report(
            db,
            target_files,
            &mapped_edges,
            &data.symbol_to_files,
            focus_query,
            output_filename,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn resolve_focus_targets(
        vector_path: &Path,
        all_files: &[String],
        db: &ContextDb,
        query: &str,
        embedder: &EmbeddingEngine,
        graph: &DependencyGraph,
        limit: usize,
    ) -> Result<Vec<String>> {
        let mut paths = Vec::new();

        for file in all_files {
            let path_obj = Path::new(file);
            let file_name = path_obj.file_name().unwrap_or_default().to_string_lossy();
            let file_stem = path_obj.file_stem().unwrap_or_default().to_string_lossy();

            if file_name.eq_ignore_ascii_case(query) || file_stem.eq_ignore_ascii_case(query) {
                if !paths.contains(file) {
                    paths.push(file.clone());
                }
            } else if let Ok(symbols) = db.get_symbols(file) {
                if symbols.iter().any(|s| s.name.eq_ignore_ascii_case(query))
                    && !paths.contains(file)
                {
                    paths.push(file.clone());
                }
            }
        }

        if paths.is_empty() {
            let store = VectorStore::open(vector_path)?;
            let query_vec = embedder.embed(query)?;
            let results = store.search(&query_vec, limit, Some(graph))?;

            if !results.is_empty() {
                let best_dist = results[0].0;
                for (dist, record) in results {
                    if dist <= best_dist + 0.25 && !paths.contains(&record.file_path) {
                        paths.push(record.file_path);
                    }
                }
            }
        }

        Ok(paths)
    }

    fn expand_graph_depth(
        initial_targets: Vec<String>,
        file_graph: &HashMap<String, HashSet<String>>,
        depth: u8,
    ) -> Vec<String> {
        let mut all_target_files: HashSet<String> = initial_targets.into_iter().collect();
        let mut current_level_files = all_target_files.clone();

        for _ in 0..depth {
            let mut next_level_files = HashSet::new();
            for current_file in &current_level_files {
                if let Some(neighbors) = file_graph.get(current_file) {
                    for neighbor in neighbors {
                        if !all_target_files.contains(neighbor) {
                            next_level_files.insert(neighbor.clone());
                        }
                    }
                }
            }
            all_target_files.extend(next_level_files.clone());
            current_level_files = next_level_files;
        }

        let mut target_files: Vec<String> = all_target_files.into_iter().collect();
        target_files.sort();
        target_files
    }

    fn format_markdown_report(
        db: &ContextDb,
        target_files: &[String],
        edges: &[(&SymbolRef, &str, Option<&str>)],
        symbol_to_files: &HashMap<String, Vec<String>>,
        focus_query: Option<&str>,
        output_filename: &str,
    ) -> Result<String> {
        let mut content = String::new();

        content.push_str(&format!("# AI Context: {}\n\n", output_filename));
        content.push_str("> Auto-generated by amdb.\n\n");

        content.push_str("## File Summaries\n\n");
        for file_path in target_files {
            let symbols = db.get_symbols(file_path)?;
            if symbols.is_empty() {
                continue;
            }

            content.push_str(&format!("### {}\n", file_path));

            for symbol in symbols {
                let doc = symbol.docstring.clone().unwrap_or_default();
                let summary = doc.lines().next().unwrap_or("").trim();

                content.push_str(&format!("- **{}** ({})", symbol.name, symbol.kind));
                if !summary.is_empty() {
                    content.push_str(&format!(": {}", summary));
                }
                content.push('\n');
                if let Some(sig) = &symbol.signature {
                    if !sig.is_empty() {
                        content.push_str(&format!("  - `{}`\n", sig));
                    }
                }
            }
            content.push('\n');
        }

        content.push_str("## Dependency Graph\n```mermaid\ngraph TD;\n");

        let target_files_set: HashSet<&String> = target_files.iter().collect();
        let is_focus = focus_query.is_some();

        let relevant_edges: Vec<(&SymbolRef, &str, Option<&str>)> = edges
            .iter()
            .filter(|(caller, _, _)| target_files_set.contains(&caller.file))
            .copied()
            .collect();

        let display_edges = if relevant_edges.len() > 100 {
            &relevant_edges[..100]
        } else {
            &relevant_edges[..]
        };

        for (caller, callee, callee_file) in display_edges {
            let mut include_edge = true;
            if is_focus {
                if let Some(files) = symbol_to_files.get(*callee) {
                    let mut callee_in_target = false;
                    for f in files {
                        if target_files_set.contains(f) {
                            callee_in_target = true;
                            break;
                        }
                    }
                    if !callee_in_target {
                        include_edge = false;
                    }
                }
            }

            if include_edge {
                let safe_caller = sanitize_node_id(&caller.file, &caller.name);
                let safe_callee = match callee_file {
                    Some(cf) => sanitize_node_id(cf, callee),
                    None => sanitize_node_id("_unresolved", callee),
                };
                if safe_caller != safe_callee {
                    content.push_str(&format!("    {} --> {};\n", safe_caller, safe_callee));
                }
            }
        }
        content.push_str("```\n");

        Ok(content)
    }
}

fn sanitize_node_id(file: &str, name: &str) -> String {
    let joined = if file.is_empty() {
        name.to_string()
    } else {
        format!("{}__{}", file, name)
    };

    joined
        .replace("::", "_")
        .replace(['.', '/', '\\', '-'], "_")
}
