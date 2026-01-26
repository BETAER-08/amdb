use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Node {
    pub file: String,
    pub name: String,
    pub kind: String,
    pub line: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DependencyGraph {
    pub nodes: HashMap<String, Node>,
    pub edges: HashMap<String, HashSet<String>>,
}

impl DependencyGraph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: HashMap::new(),
        }
    }

    #[allow(dead_code)]
    pub fn add_node(&mut self, file: &str, name: &str, kind: &str, line: usize) {
        let key = format!("{}::{}", file, name);
        self.nodes.insert(key, Node {
            file: file.to_string(),
            name: name.to_string(),
            kind: kind.to_string(),
            line,
        });
    }

    pub fn add_edge(&mut self, file: &str, caller: &str, callee: &str) {
        let key = format!("{}::{}", file, caller);
        self.edges
            .entry(key)
            .or_insert_with(HashSet::new)
            .insert(callee.to_string());
    }

    pub fn debug_print(&self) {
        for (caller, callees) in &self.edges {
            println!("{} calls:", caller);
            for callee in callees {
                println!("  -> {}", callee);
            }
        }
    }
}