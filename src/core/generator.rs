use anyhow::Result;
use console::style;
use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use crate::core::embedding::EmbeddingEngine;
use crate::core::graph::DependencyGraph;
use crate::core::symbol::SymbolRef;
use crate::core::vector_store::VectorStore;
use crate::db::ContextDb;

pub struct ContextGenerator;

impl ContextGenerator {
    pub async fn generate(focus_query: Option<String>, depth: u8) -> Result<()> {
        let db_dir = Path::new(".database");
        let output_dir = Path::new(".amdb");

        if !output_dir.exists() {
            fs::create_dir(output_dir)?;
        }

        if !db_dir.exists() {
            eprintln!("{}", style("Error: Database not found. Run 'amdb init' first.").red());
            std::process::exit(1);
        }

        let db = ContextDb::open(db_dir)?;
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
                    symbol_to_files.entry(sym.name).or_default().push(file.clone());
                }
            }
        }

        let target_files: Vec<String>;
        let output_filename: String;

        if let Some(query) = &focus_query {
            let safe_name = query.replace(" ", "-").replace("/", "-").to_lowercase();
            output_filename = format!("{}.md", safe_name);

            println!("{}", style(format!("Filtering context for: '{}' with depth {}...", query, depth)).cyan());

            let embedder = EmbeddingEngine::new()?;
            let paths = Self::resolve_focus_targets(db_dir, &all_files, &db, query, &embedder, &graph).await?;

            if paths.is_empty() {
                println!("{}", style("No matches found. Falling back to full context.").yellow());
                target_files = all_files;
            } else {
                let mut file_graph: HashMap<String, HashSet<String>> = HashMap::new();
                for edge in &all_edges {
                    let caller_file = &edge.caller.file;
                    if let Some(callee_files) = symbol_to_files.get(&edge.callee) {
                        for callee_file in callee_files {
                            if caller_file != callee_file {
                                file_graph.entry(caller_file.clone()).or_default().insert(callee_file.clone());
                            }
                        }
                    }
                }
                target_files = Self::expand_graph_depth(paths, &file_graph, depth);
            }
        } else {
            output_filename = "context.md".to_string();
            println!("{}", style("Generating full project context...").blue());
            target_files = all_files;
        }

        let mapped_edges: Vec<(&SymbolRef, &str)> = all_edges.iter().map(|e| (&e.caller, e.callee.as_str())).collect();

        let content = Self::format_markdown_report(
            &db,
            &target_files,
            &mapped_edges,
            &symbol_to_files,
            focus_query.as_deref(),
            &output_filename,
        )?;

        let output_path = output_dir.join(&output_filename);
        let mut file = File::create(&output_path)?;
        file.write_all(content.as_bytes())?;

        println!("{}", style(format!("Generated: {}", output_path.display())).green().bold());
        Ok(())
    }

    async fn resolve_focus_targets(
        db_dir: &Path,
        all_files: &[String],
        db: &ContextDb,
        query: &str,
        embedder: &EmbeddingEngine,
        graph: &DependencyGraph,
    ) -> Result<Vec<String>> {
        let mut paths = Vec::new();

        for file in all_files {
            let path_obj = Path::new(file);
            let file_name = path_obj.file_name().unwrap_or_default().to_string_lossy();
            let file_stem = path_obj.file_stem().unwrap_or_default().to_string_lossy();

            if file_name.eq_ignore_ascii_case(query) || file_stem.eq_ignore_ascii_case(query) {
                if !paths.contains(file) { paths.push(file.clone()); }
            } else if let Ok(symbols) = db.get_symbols(file) {
                if symbols.iter().any(|s| s.name.eq_ignore_ascii_case(query))
                    && !paths.contains(file)
                {
                    paths.push(file.clone());
                }
            }
        }

        if paths.is_empty() {
            let vector_path = db_dir.join("vector");
            let store = VectorStore::open(&vector_path)?;
            let query_vec = embedder.embed(query)?;
            let results = store.search(&query_vec, 10, Some(graph))?;

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
        edges: &[(&SymbolRef, &str)],
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
            if symbols.is_empty() { continue; }

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

        let relevant_edges: Vec<(&SymbolRef, &str)> = edges
            .iter()
            .filter(|(caller, _)| target_files_set.contains(&caller.file))
            .copied()
            .collect();

        let display_edges = if relevant_edges.len() > 100 {
            &relevant_edges[..100]
        } else {
            &relevant_edges[..]
        };

        for (caller, callee) in display_edges {
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
                let safe_caller = sanitize_mermaid_id(&caller.name);
                let safe_callee = sanitize_mermaid_id(callee);
                if safe_caller != safe_callee {
                    content.push_str(&format!("    {} --> {};\n", safe_caller, safe_callee));
                }
            }
        }
        content.push_str("```\n");

        Ok(content)
    }
}

fn sanitize_mermaid_id(name: &str) -> String {
    name.replace("::", "_").replace(['.', '/', '\\'], "_")
}
