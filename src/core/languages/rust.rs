use super::{has_child_of_kind, signature_before_body, SymbolEnricher};
use tree_sitter::{Language, Node};

pub fn language() -> Language {
    tree_sitter_rust::LANGUAGE.into()
}

pub struct RustEnricher;

pub static ENRICHER: RustEnricher = RustEnricher;

impl SymbolEnricher for RustEnricher {
    fn is_public(&self, node: Node, _src: &str) -> bool {
        has_child_of_kind(node, "visibility_modifier")
    }

    fn signature(&self, node: Node, src: &str) -> Option<String> {
        signature_before_body(node, src)
    }
}

pub const QUERY: &str = r#"
    (function_item name: (identifier) @Function)
    (struct_item name: (type_identifier) @Struct)
    (impl_item type: (type_identifier) @Class)
    (call_expression function: [
        (identifier) @Call
        (field_expression field: (field_identifier) @Call)
        (scoped_identifier name: (identifier) @Call)
    ])
"#;
