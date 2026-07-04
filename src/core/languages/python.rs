use super::{signature_before_body, SymbolEnricher};
use tree_sitter::{Language, Node};

pub fn language() -> Language {
    tree_sitter_python::LANGUAGE.into()
}

pub struct PythonEnricher;

pub static ENRICHER: PythonEnricher = PythonEnricher;

impl SymbolEnricher for PythonEnricher {
    fn is_public(&self, node: Node, src: &str) -> bool {
        node.child_by_field_name("name")
            .and_then(|n| n.utf8_text(src.as_bytes()).ok())
            .map(|name| !name.starts_with('_'))
            .unwrap_or(true)
    }

    fn signature(&self, node: Node, src: &str) -> Option<String> {
        signature_before_body(node, src)
    }
}

pub const QUERY: &str = r#"
    (function_definition
      name: (identifier) @Function
      body: (block . (expression_statement (string) @doc)?)
    )
    (class_definition
      name: (identifier) @Class
      body: (block . (expression_statement (string) @doc)?)
    )
    (call
      function: [
        (attribute object: (_) attribute: (identifier) @Call)
        (identifier) @Call
      ]
    )
"#;
