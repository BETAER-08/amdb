use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
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

pub struct SymbolResolver {
    by_name: HashMap<String, HashSet<String>>,
}

impl SymbolResolver {
    pub fn new() -> Self {
        Self {
            by_name: HashMap::new(),
        }
    }

    pub fn register(&mut self, file: &str, name: &str) {
        self.by_name
            .entry(name.to_string())
            .or_default()
            .insert(file.to_string());
    }

    pub fn resolve(&self, caller_file: &str, callee_name: &str) -> Option<String> {
        let files = self.by_name.get(callee_name)?;

        if files.contains(caller_file) {
            return Some(caller_file.to_string());
        }

        if files.len() == 1 {
            files.iter().next().cloned()
        } else {
            None
        }
    }
}

impl Default for SymbolResolver {
    fn default() -> Self {
        Self::new()
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
