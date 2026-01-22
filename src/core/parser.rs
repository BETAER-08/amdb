use anyhow::Result;
use tree_sitter::{Parser, Query, QueryCursor, Node};
use serde::{Serialize, Deserialize};
pub use super::languages::SupportedLanguage;
use super::graph::DependencyGraph;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeSymbol {
    pub kind: String,
    pub name: String,
    pub line: usize,
    pub docstring: Option<String>,
    pub signature: Option<String>,
    pub is_public: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct CodeRoute {
    pub method: String,
    pub path: String,
    pub handler: Option<String>,
}

pub struct CodeParser {
    parser: Parser,
    language: SupportedLanguage,
}

impl CodeParser {
    pub fn new(lang: SupportedLanguage) -> Result<Self> {
        let mut parser = Parser::new();
        let ts_lang = lang.get_language();
        parser.set_language(&ts_lang)?;
        Ok(Self { parser, language: lang })
    }

    pub fn parse(&mut self, file_path: &str, code: &str) 
        -> Result<(Vec<CodeSymbol>, DependencyGraph, Vec<CodeRoute>)> 
    {
        let tree = self.parser.parse(code, None)
            .ok_or_else(|| anyhow::anyhow!("Parsing failed"))?;
        
        let ts_lang = self.language.get_language();
        let query_str = self.language.get_query();
        let query = Query::new(&ts_lang, query_str)?;
        
        let mut cursor = QueryCursor::new();
        let mut symbols = Vec::new();
        let mut routes = Vec::new();
        let mut graph = DependencyGraph::new();

        let matches = cursor.matches(&query, tree.root_node(), code.as_bytes());
        
        for m in matches {
            let mut kind = String::new();
            let mut name = String::new();
            let mut docstring = None;
            let mut signature = None;
            let mut is_public = false;
            let mut route_method = None;
            let mut route_path = None;

            for capture in m.captures {
                let capture_name = query.capture_names()[capture.index as usize].as_str();
                let text = capture.node.utf8_text(code.as_bytes())?.to_string();

                match capture_name {
                    "Function" | "Class" | "Interface" | "Method" | "Struct" => {
                        kind = capture_name.to_string();
                        name = text;
                    },
                    "Call" => {
                        if let Some(caller) = self.find_parent_function(capture.node, code) {
                            graph.add_edge(file_path, &caller, &text);
                        }
                    },
                    "doc" => docstring = Some(text),
                    "sig" => signature = Some(text),
                    "pub" => is_public = true,
                    "route_method" => route_method = Some(text.replace("\"", "").to_lowercase()),
                    "route_path" => route_path = Some(text.replace("\"", "")),
                    _ => {}
                }
            }

            if !name.is_empty() && !kind.is_empty() {
                symbols.push(CodeSymbol {
                    kind, name, line: m.pattern_index, 
                    docstring, signature, is_public
                });
            }

            if let (Some(method), Some(path)) = (route_method, route_path) {
                routes.push(CodeRoute {
                    method, path, handler: None 
                });
            }
        }
        
        Ok((symbols, graph, routes))
    }

    fn find_parent_function(&self, node: Node, code: &str) -> Option<String> {
        let mut curr = node.parent();
        while let Some(n) = curr {
            let kind = n.kind();
            if kind.contains("function") || kind.contains("method") {
                if let Some(name_node) = n.child_by_field_name("name") {
                     return name_node.utf8_text(code.as_bytes()).ok().map(|s| s.to_string());
                }
            }
            curr = n.parent();
        }
        None
    }
}