use crate::core::symbol::SymbolRef;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Node {
    pub sym: SymbolRef,
    pub kind: String,
    pub line: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DependencyGraph {
    pub nodes: HashMap<SymbolRef, Node>,
    pub edges: HashMap<SymbolRef, HashSet<String>>,
    pub reverse_edges: HashMap<String, HashSet<SymbolRef>>,
}

impl DependencyGraph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: HashMap::new(),
            reverse_edges: HashMap::new(),
        }
    }

    #[allow(dead_code)]
    pub fn add_node(&mut self, file: &str, name: &str, kind: &str, line: usize) {
        let sym = SymbolRef::new(file, name);
        self.nodes.insert(
            sym.clone(),
            Node {
                sym,
                kind: kind.to_string(),
                line,
            },
        );
    }

    pub fn add_edge(&mut self, file: &str, caller: &str, callee: &str) {
        let sym = SymbolRef::new(file, caller);
        self.edges
            .entry(sym.clone())
            .or_default()
            .insert(callee.to_string());
        self.reverse_edges
            .entry(callee.to_string())
            .or_default()
            .insert(sym);
    }

    #[allow(dead_code)]
    pub fn debug_print(&self) {
        for (caller, callees) in &self.edges {
            println!("{} calls:", caller.to_key());
            for callee in callees {
                println!("  -> {}", callee);
            }
        }
    }
}
