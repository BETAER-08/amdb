use std::collections::{HashMap, HashSet};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Node {
    pub name: String,
    pub kind: String,
    pub file: String,
    pub line: usize,
}

#[derive(Debug, Serialize)]
pub struct DependencyGraph {
    // [수정] pub 키워드 추가 (외부 공개)
    pub nodes: HashMap<String, Node>,
    // [수정] pub 키워드 추가 (외부 공개)
    pub edges: HashMap<String, HashSet<String>>,
}

impl DependencyGraph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, file: &str, name: &str, kind: &str, line: usize) {
        let key = format!("{}::{}", file, name);
        self.nodes.insert(key, Node {
            name: name.to_string(),
            kind: kind.to_string(),
            file: file.to_string(),
            line,
        });
    }

    pub fn add_edge(&mut self, caller_file: &str, caller_name: &str, callee_name: &str) {
        let caller_key = format!("{}::{}", caller_file, caller_name);
        self.edges.entry(caller_key)
            .or_default()
            .insert(callee_name.to_string());
    }

    pub fn debug_print(&self) {
        println!("[Graph] Total Nodes: {}, Total Relations: {}", self.nodes.len(), self.edges.len());
        for (caller, callees) in &self.edges {
            println!("  {} calls: {:?}", caller, callees);
        }
    }
}