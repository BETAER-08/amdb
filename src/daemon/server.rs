use axum::{
    extract::{Path as AxumPath, State},
    routing::get,
    Json, Router,
};
use serde::Serialize;
use std::sync::Arc;
use tokio::net::TcpListener;
use crate::db::ContextDb;
use crate::core::parser::CodeSymbol;

#[derive(Clone)]
struct AppState {
    db_path: String,
}

#[derive(Serialize)]
struct ContextResponse {
    file: String,
    symbols: Vec<CodeSymbol>,
    relationships: Vec<crate::db::query::Relationship>,
}

pub async fn start_server() -> anyhow::Result<()> {
    let state = Arc::new(AppState {
        db_path: ".amdb".to_string(),
    });

    let app = Router::new()
        .route("/context/*file_path", get(get_context))
        .with_state(state);

    let listener = TcpListener::bind("0.0.0.0:3000").await?;
    println!("Server running on http://0.0.0.0:3000");
    
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

    Json(ContextResponse {
        file: file_path,
        symbols,
        relationships,
    })
}