use anyhow::Result;
use console::style;
use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use crate::core::embedding::EmbeddingEngine;
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
            println!("{}", style("Error: Database not found. Run 'amdb init' first.").red());
            return Ok(());
        }

        let db = ContextDb::open(db_dir)?;
        let all_edges = db.get_all_relationships()?;
        let all_files = db.get_all_files()?;
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

            let paths = Self::resolve_focus_targets(db_dir, &all_files, &db, query).await?;

            if paths.is_empty() {
                println!("{}", style("No matches found. Falling back to full context.").yellow());
                target_files = all_files;
            } else {
                let mut file_graph: HashMap<String, HashSet<String>> = HashMap::new();
                for edge in &all_edges {
                    if let Some(caller_file) = edge.caller.split("::").next() {
                        if let Some(callee_files) = symbol_to_files.get(&edge.callee) {
                            for callee_file in callee_files {
                                if caller_file != callee_file {
                                    file_graph.entry(caller_file.to_string()).or_default().insert(callee_file.clone());
                                    file_graph.entry(callee_file.clone()).or_default().insert(caller_file.to_string());
                                }
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

        let mapped_edges: Vec<(&str, &str)> = all_edges.iter().map(|e| (e.caller.as_str(), e.callee.as_str())).collect();

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
    ) -> Result<Vec<String>> {
        let mut paths = Vec::new();

        for file in all_files {
            let path_obj = Path::new(file);
            let file_name = path_obj.file_name().unwrap_or_default().to_string_lossy();
            let file_stem = path_obj.file_stem().unwrap_or_default().to_string_lossy();

            if file_name.eq_ignore_ascii_case(query) || file_stem.eq_ignore_ascii_case(query) {
                if !paths.contains(file) { paths.push(file.clone()); }
            } else if let Ok(symbols) = db.get_symbols(file) {
                if symbols.iter().any(|s| s.name.eq_ignore_ascii_case(query)) {
                    if !paths.contains(file) { paths.push(file.clone()); }
                }
            }
        }

        if paths.is_empty() {
            let vector_path = db_dir.join("vector");
            let store = VectorStore::open(&vector_path)?;
            let embedder = EmbeddingEngine::new()?;
            let query_vec = embedder.embed(query)?;
            let results = store.search(&query_vec, 10, None)?;

            if !results.is_empty() {
                let best_dist = results[0].0;
                for (dist, record) in results {
                    if dist <= best_dist + 0.25 {
                        if !paths.contains(&record.file_path) {
                            paths.push(record.file_path);
                        }
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
        edges: &[(&str, &str)],
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
            }
            content.push('\n');
        }

        content.push_str("## Dependency Graph\n```mermaid\ngraph TD;\n");

        let target_files_set: HashSet<&String> = target_files.iter().collect();
        let mut edge_count = 0;
        let is_focus = focus_query.is_some();

        for (caller, callee) in edges {
            if edge_count > 100 { break; }

            if let Some(caller_file) = caller.split("::").next() {
                let caller_file_str = caller_file.to_string();
                if is_focus && !target_files_set.contains(&caller_file_str) {
                    continue;
                }

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
                    let safe_caller = caller.replace("::", "_").replace(".", "_").replace("/", "_").replace("\\", "_");
                    let safe_callee = callee.replace("::", "_").replace(".", "_").replace("/", "_").replace("\\", "_");
                    if safe_caller != safe_callee {
                        content.push_str(&format!("    {} --> {};\n", safe_caller, safe_callee));
                        edge_count += 1;
                    }
                }
            }
        }
        content.push_str("```\n");

        Ok(content)
    }
}