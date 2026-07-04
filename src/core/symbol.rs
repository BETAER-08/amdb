use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SymbolRef {
    pub file: String,
    pub name: String,
}

impl SymbolRef {
    pub fn new(file: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            file: file.into(),
            name: name.into(),
        }
    }

    pub fn to_key(&self) -> String {
        format!("{}::{}", self.file, self.name)
    }
}

pub fn normalize_path(root: &str, path: &Path) -> String {
    let root_path = Path::new(root);
    let canon_root = std::fs::canonicalize(root_path).unwrap_or_else(|_| root_path.to_path_buf());
    let candidate = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());

    let relative = candidate
        .strip_prefix(&canon_root)
        .or_else(|_| candidate.strip_prefix(root_path))
        .unwrap_or(candidate.as_path());

    let normalized = relative.to_string_lossy().replace('\\', "/");
    let trimmed = normalized.trim_start_matches("./");

    if trimmed.is_empty() {
        normalized
    } else {
        trimmed.to_string()
    }
}
