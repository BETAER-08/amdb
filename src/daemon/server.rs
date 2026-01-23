use axum::{
    extract::{Path as AxumPath, State, Query},
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use crate::db::ContextDb;
use crate::core::parser::{CodeSymbol, CodeWarning};
use crate::core::embedding::EmbeddingEngine;
use crate::core::vector_store::VectorStore;
use crate::core::config::AppConfig;

#[derive(Clone)]
struct AppState {
    db_path: String,
    vector_store: Arc<Mutex<VectorStore>>,
    embedder: Arc<EmbeddingEngine>,
}

#[derive(Serialize)]
struct ContextResponse {
    file: String,
    symbols: Vec<CodeSymbol>,
    relationships: Vec<crate::db::query::Relationship>,
    warnings: Vec<CodeWarning>,
}

#[derive(Deserialize)]
struct SearchParams {
    q: String,
}

#[derive(Serialize)]
struct SearchResponse {
    score: f64,
    file: String,
    id: String,
    text: String,
}

pub async fn start_server() -> anyhow::Result<()> {
    let config = AppConfig::load();
    let port = config.server_port;

    let vector_path = std::path::Path::new(".amdb/vector");
    let vector_store = VectorStore::load(vector_path).unwrap_or(VectorStore::new());
    let embedder = EmbeddingEngine::new()?;

    let state = Arc::new(AppState {
        db_path: ".amdb".to_string(),
        vector_store: Arc::new(Mutex::new(vector_store)),
        embedder: Arc::new(embedder),
    });

    let app = Router::new()
        .route("/context/*file_path", get(get_context))
        .route("/search", get(handle_search))
        .with_state(state);

    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&addr).await?;
    println!("Server running on http://{}", addr);
    
    axum::serve(listener, app).await?;
    Ok(())
}

async fn get_context(
    State(state): State<Arc<AppState>>,
    AxumPath(file_path): AxumPath<String>,
) -> Json<ContextResponse> {
    let path = std::path::Path::new(&state.db_path);
    let db = ContextDb::open(path).expect("Failed to open DB");
    
    let symbols = db.get_symbols(&file_path).unwrap_or_default();
    let relationships = db.get_relationships(&file_path).unwrap_or_default();
    let warnings = db.get_warnings(&file_path).unwrap_or_default();

    Json(ContextResponse {
        file: file_path,
        symbols,
        relationships,
        warnings,
    })
}

async fn handle_search(
    State(state): State<Arc<AppState>>,
    Query(params): Query<SearchParams>,
) -> Json<Vec<SearchResponse>> {
    let query_vec = match state.embedder.embed(&params.q) {
        Ok(v) => v,
        Err(_) => return Json(vec![]),
    };

    let store = state.vector_store.lock().unwrap();
    let results = store.search(&query_vec, 5);

    let response = results.into_iter()
        .map(|(score, record)| SearchResponse { 
            score, 
            file: record.file_path,
            id: record.id,
            text: record.text 
        })
        .collect();

    Json(response)
}