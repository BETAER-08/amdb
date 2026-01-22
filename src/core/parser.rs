use anyhow::Result;
use tree_sitter::{Parser, Query, QueryCursor, Node};
use serde::Serialize; // [추가]

// [수정] 아래 줄에 Serialize 추가
pub use super::languages::SupportedLanguage;
use super::graph::DependencyGraph;

// [수정] #[derive(Debug, Clone, Serialize)] 로 변경
#[derive(Debug, Clone, Serialize)]
pub struct CodeSymbol {
    pub kind: String,
    pub name: String,
    pub line: usize,
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

    pub fn parse(&mut self, file_path: &str, code: &str) -> Result<(Vec<CodeSymbol>, DependencyGraph)> {
        let tree = self.parser.parse(code, None)
            .ok_or_else(|| anyhow::anyhow!("Parsing failed"))?;
        
        let ts_lang = self.language.get_language();
        let query_str = self.language.get_query();
        let query = Query::new(&ts_lang, query_str)?;
        
        let mut cursor = QueryCursor::new();
        let mut symbols = Vec::new();
        let mut graph = DependencyGraph::new();

        for m in cursor.matches(&query, tree.root_node(), code.as_bytes()) {
            for capture in m.captures {
                let kind = query.capture_names()[capture.index as usize].to_string();
                let name = capture.node.utf8_text(code.as_bytes())?.to_string();
                let line = capture.node.start_position().row + 1;

                if kind == "Call" {
                    if let Some(caller_name) = self.find_parent_function(capture.node, code) {
                        graph.add_edge(file_path, &caller_name, &name);
                    }
                } else {
                    symbols.push(CodeSymbol { kind: kind.clone(), name: name.clone(), line });
                    graph.add_node(file_path, &name, &kind, line);
                }
            }
        }
        
        Ok((symbols, graph))
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