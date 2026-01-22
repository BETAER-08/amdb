use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};
use crate::db::ContextDb;

#[derive(Deserialize, Debug)]
struct JsonRpcRequest {
    jsonrpc: String,
    method: String,
    params: Option<Value>,
    id: Option<Value>,
}

#[derive(Serialize, Debug)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: Option<Value>,
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

#[derive(Serialize, Debug)]
struct JsonRpcError {
    code: i32,
    message: String,
}

pub fn run_stdio_server() -> Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() { continue; }

        let req: JsonRpcRequest = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(_) => continue, 
        };

        let result = match req.method.as_str() {
            "initialize" => handle_initialize(),
            "notifications/initialized" => Ok(json!(null)),
            "resources/list" => handle_list_resources(),
            "resources/read" => handle_read_resource(req.params),
            "ping" => Ok(json!({})),
            _ => Err(JsonRpcError { code: -32601, message: "Method not found".into() }),
        };

        if let Some(id) = req.id {
            let response = match result {
                Ok(res) => JsonRpcResponse {
                    jsonrpc: "2.0".into(),
                    id: Some(id),
                    result: Some(res),
                    error: None,
                },
                Err(e) => JsonRpcResponse {
                    jsonrpc: "2.0".into(),
                    id: Some(id),
                    result: None,
                    error: Some(e),
                },
            };

            let response_str = serde_json::to_string(&response)?;
            writeln!(stdout, "{}", response_str)?;
            stdout.flush()?;
        }
    }

    Ok(())
}

fn handle_initialize() -> Result<Value, JsonRpcError> {
    Ok(json!({
        "protocolVersion": "0.1.0",
        "capabilities": {
            "resources": {} 
        },
        "serverInfo": {
            "name": "ctx-mcp",
            "version": "0.1.0"
        }
    }))
}

fn handle_list_resources() -> Result<Value, JsonRpcError> {
    let resources = json!([
        {
            "uri": "ctx://context_summary",
            "name": "Project Context Summary",
            "description": "Global summary of the project structure"
        }
    ]);

    Ok(json!({ "resources": resources }))
}

fn handle_read_resource(params: Option<Value>) -> Result<Value, JsonRpcError> {
    let uri = params.and_then(|p| p.get("uri").cloned())
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .ok_or(JsonRpcError { code: -32602, message: "Missing uri parameter".into() })?;

    if !uri.starts_with("ctx://") {
        return Err(JsonRpcError { code: -32602, message: "Invalid URI scheme".into() });
    }

    let file_path = uri.replace("ctx://", ""); 
    
    let ctx_path = std::path::Path::new(".ctx");
    let db = ContextDb::open(ctx_path).map_err(|_| JsonRpcError { 
        code: -32000, message: "Failed to open .ctx/store.db".into() 
    })?;

    let symbols = db.get_symbols(&file_path).unwrap_or_default();
    let relationships = db.get_relationships(&file_path).unwrap_or_default();

    let content_json = json!({
        "file": file_path,
        "symbols": symbols,
        "relationships": relationships
    });

    Ok(json!({
        "contents": [{
            "uri": uri,
            "mimeType": "application/json",
            "text": content_json.to_string()
        }]
    }))
}