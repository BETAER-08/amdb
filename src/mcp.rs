use crate::core::config::Config;
use crate::core::generator::ContextGenerator;
use crate::db::ContextDb;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, ContentBlock, ErrorData};
use rmcp::{tool, tool_handler, tool_router, ServerHandler, ServiceExt};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;
use std::path::PathBuf;

#[derive(Deserialize, JsonSchema)]
pub struct GetContextParams {
    #[schemars(
        description = "Accepted for parity with `amdb generate`; the full-project overview does not vary with depth. Default 1."
    )]
    pub depth: Option<u8>,
}

#[derive(Deserialize, JsonSchema)]
pub struct FocusParams {
    #[schemars(
        description = "What to focus on: a symbol name, a file name, or a natural-language description of a feature or concept."
    )]
    pub query: String,
    #[schemars(
        description = "How many dependency-graph hops around the matched files to include. Default 1."
    )]
    pub depth: Option<u8>,
    #[schemars(
        description = "Maximum number of semantic vector-search matches to consider when no exact name match exists. Default 10."
    )]
    pub limit: Option<u32>,
}

#[derive(Deserialize, JsonSchema)]
pub struct GetSymbolParams {
    #[schemars(description = "Exact symbol name to look up (case-sensitive).")]
    pub name: String,
    #[schemars(
        description = "Optional file path (relative to the project root, as shown by amdb_get_context) to restrict the lookup to when the name is defined in several files."
    )]
    pub file: Option<String>,
}

pub struct AmdbServer;

struct Index {
    db: ContextDb,
    vector_path: PathBuf,
    warning: Option<String>,
}

fn internal(e: anyhow::Error) -> ErrorData {
    ErrorData::internal_error(format!("{:#}", e), None)
}

impl AmdbServer {
    fn open_index(&self) -> Result<Index, CallToolResult> {
        let config = Config::load(".");
        let db_dir = config.db_dir(".");

        if !db_dir.join("context.db").exists() {
            return Err(CallToolResult::error(vec![ContentBlock::text(format!(
                "No amdb index found at {}. Run `amdb init` in the project root first, then retry.",
                db_dir.display()
            ))]));
        }

        let db = ContextDb::open(&db_dir).map_err(|e| {
            CallToolResult::error(vec![ContentBlock::text(format!(
                "Failed to open the amdb index at {}: {}. Run `amdb init` to rebuild it.",
                db_dir.display(),
                e
            ))])
        })?;

        let warning = db.needs_reindex.then(|| {
            "WARNING: the index uses a legacy schema and may be stale. Run `amdb init` to rebuild it before relying on these results.".to_string()
        });

        Ok(Index {
            db,
            vector_path: config.vector_dir("."),
            warning,
        })
    }

    fn ok_with_warning(warning: Option<String>, payload: String) -> CallToolResult {
        let mut content = Vec::new();
        if let Some(w) = warning {
            content.push(ContentBlock::text(w));
        }
        content.push(ContentBlock::text(payload));
        CallToolResult::success(content)
    }
}

#[tool_router]
impl AmdbServer {
    #[tool(
        description = "Return a full overview of the indexed project: every file with its symbols (name, kind, signature) plus the cross-file dependency graph in mermaid form. Use this first to orient yourself in the codebase before drilling into specific files. Results come from the pre-built local amdb index created by `amdb init`; they reflect the last index run, not the live file system."
    )]
    async fn amdb_get_context(
        &self,
        params: Parameters<GetContextParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let _ = params.0.depth;
        let index = match self.open_index() {
            Ok(i) => i,
            Err(e) => return Ok(e),
        };

        let data = ContextGenerator::load_graph_data(&index.db).map_err(internal)?;
        let report =
            ContextGenerator::render_report(&index.db, &data, &data.all_files, None, "context.md")
                .map_err(internal)?;

        Ok(Self::ok_with_warning(index.warning, report))
    }

    #[tool(
        description = "Return project context narrowed to a query: files whose name or symbols match it exactly, or, failing that, the semantically closest files found by local vector search with dependency-graph boosting, expanded by `depth` dependency hops. Use this when you need context about one feature, file, or concept instead of the whole project. Results come from the pre-built local amdb index and its embeddings; run `amdb init` to refresh them."
    )]
    async fn amdb_focus(
        &self,
        params: Parameters<FocusParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let p = params.0;
        let index = match self.open_index() {
            Ok(i) => i,
            Err(e) => return Ok(e),
        };

        let depth = p.depth.unwrap_or(1);
        let limit = p.limit.unwrap_or(10) as usize;

        let data = ContextGenerator::load_graph_data(&index.db).map_err(internal)?;
        let targets = ContextGenerator::compute_focus_files(
            &index.db,
            &index.vector_path,
            &data,
            &p.query,
            depth,
            limit,
        )
        .map_err(internal)?;

        match targets {
            None => Ok(Self::ok_with_warning(
                index.warning,
                format!(
                    "No files in the index matched '{}' by name, symbol, or semantic similarity.",
                    p.query
                ),
            )),
            Some(files) => {
                let filename = ContextGenerator::focus_filename(&p.query);
                let report = ContextGenerator::render_report(
                    &index.db,
                    &data,
                    &files,
                    Some(&p.query),
                    &filename,
                )
                .map_err(internal)?;
                Ok(Self::ok_with_warning(index.warning, report))
            }
        }
    }

    #[tool(
        description = "Look up a symbol by exact name and return every definition in the index as a JSON array. Each match has: file, name, kind, line, signature, is_public, callers (symbols that call it, with their files), and callees (symbols it calls; each callee's file comes from the resolver and is null when the definition is ambiguous or unknown). When a name is defined in multiple files all matches are returned; pass `file` to restrict to one. Use this to jump to a definition or trace call relationships. Results come from the pre-built local amdb index created by `amdb init`."
    )]
    async fn amdb_get_symbol(
        &self,
        params: Parameters<GetSymbolParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let p = params.0;
        let index = match self.open_index() {
            Ok(i) => i,
            Err(e) => return Ok(e),
        };

        let mut symbols = index
            .db
            .get_symbols_by_name(&p.name)
            .map_err(|e| internal(e.into()))?;

        if let Some(file) = &p.file {
            symbols.retain(|(f, _)| f == file);
        }

        let mut matches = Vec::new();
        for (file, sym) in symbols {
            let callers: Vec<serde_json::Value> = index
                .db
                .get_callers_of(&p.name, &file)
                .map_err(|e| internal(e.into()))?
                .into_iter()
                .map(|r| json!({ "name": r.name, "file": r.file }))
                .collect();

            let callees: Vec<serde_json::Value> = index
                .db
                .get_callees_of(&file, &sym.name)
                .map_err(|e| internal(e.into()))?
                .into_iter()
                .map(|(name, callee_file)| json!({ "name": name, "file": callee_file }))
                .collect();

            matches.push(json!({
                "file": file,
                "name": sym.name,
                "kind": sym.kind,
                "line": sym.line,
                "signature": sym.signature,
                "is_public": sym.is_public,
                "callers": callers,
                "callees": callees,
            }));
        }

        let payload = serde_json::to_string_pretty(&serde_json::Value::Array(matches))
            .map_err(|e| internal(e.into()))?;

        Ok(Self::ok_with_warning(index.warning, payload))
    }
}

#[tool_handler(
    name = "amdb",
    version = "0.7.0",
    instructions = "amdb serves a pre-built local index of this codebase (SQLite + embeddings, built by `amdb init`). Use amdb_get_context for a project overview, amdb_focus for context about one feature or file, and amdb_get_symbol to find definitions and call relationships. All data is local; nothing leaves the machine. If tools report a missing or stale index, ask the user to run `amdb init`."
)]
impl ServerHandler for AmdbServer {}

pub async fn serve_stdio() -> anyhow::Result<()> {
    let service = AmdbServer.serve(rmcp::transport::stdio()).await?;
    service.waiting().await?;
    Ok(())
}
