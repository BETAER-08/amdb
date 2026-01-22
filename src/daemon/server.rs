use axum::{
    extract::Query,
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use crate::db::{ContextDb, query::Relationship}; // [수정] Relationship 추가
use crate::core::parser::CodeSymbol;

#[derive(Deserialize)]
struct ContextParams {
    file: String,
}

// [신규] 최종 응답 포맷 (심볼 + 관계)
#[derive(Serialize)]
struct ContextResponse {
    file: String,
    symbols: Vec<CodeSymbol>,
    relationships: Vec<Relationship>,
}

pub async fn start_server() -> anyhow::Result<()> {
    let app = Router::new()
        .route("/context", get(get_context));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Server running on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

// [변경] 반환 타입이 Json<Vec<CodeSymbol>> -> Json<ContextResponse> 로 변경됨
async fn get_context(Query(params): Query<ContextParams>) -> Json<ContextResponse> {
    let ctx_path = std::path::Path::new(".ctx");
    
    // DB에서 심볼과 관계를 모두 가져옴
    let (symbols, relationships) = match ContextDb::open(ctx_path) {
        Ok(db) => {
            let syms = db.get_symbols(&params.file).unwrap_or_default();
            let rels = db.get_relationships(&params.file).unwrap_or_default();
            (syms, rels)
        },
        Err(_) => (vec![], vec![]),
    };

    Json(ContextResponse {
        file: params.file,
        symbols,
        relationships,
    })
}