use axum::{
    routing::get,
    Router,
    extract::Query,
    Json,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::path::Path;
use crate::db::ContextDb;

#[derive(Deserialize)]
pub struct ContextQuery {
    file: String,
}

#[derive(Serialize)]
pub struct ContextResponse {
    file: String,
    symbols: Vec<SymbolData>,
}

#[derive(Serialize)]
pub struct SymbolData {
    kind: String,
    name: String,
    line: usize,
}

pub async fn start_server() -> anyhow::Result<()> {
    let app = Router::new()
        .route("/context", get(get_context));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("MCP Server running at http://{}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn get_context(Query(params): Query<ContextQuery>) -> Json<ContextResponse> {
    let ctx_path = Path::new(".ctx");
    let symbols = match ContextDb::open(ctx_path) {
        Ok(db) => db.get_symbols(&params.file).unwrap_or_default(),
        Err(_) => vec![],
    };

    let data = symbols.into_iter().map(|s| SymbolData {
        kind: s.kind,
        name: s.name,
        line: s.line,
    }).collect();

    Json(ContextResponse {
        file: params.file,
        symbols: data,
    })
}