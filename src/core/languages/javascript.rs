use super::{signature_before_body, SymbolEnricher};
use tree_sitter::{Language, Node};

pub struct TypeScriptEnricher;

pub static TS_ENRICHER: TypeScriptEnricher = TypeScriptEnricher;

impl SymbolEnricher for TypeScriptEnricher {
    fn is_public(&self, node: Node, src: &str) -> bool {
        let mut cursor = node.walk();
        let has_private_modifier = node.children(&mut cursor).any(|c| {
            c.kind() == "accessibility_modifier"
                && matches!(c.utf8_text(src.as_bytes()), Ok("private") | Ok("protected"))
        });

        let name_is_private = node
            .child_by_field_name("name")
            .map(|n| n.kind() == "private_property_identifier")
            .unwrap_or(false);

        !has_private_modifier && !name_is_private
    }

    fn signature(&self, node: Node, src: &str) -> Option<String> {
        signature_before_body(node, src)
    }
}

pub fn language_js() -> Language {
    tree_sitter_javascript::LANGUAGE.into()
}

pub fn language_ts() -> Language {
    tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()
}

pub fn language_tsx() -> Language {
    tree_sitter_typescript::LANGUAGE_TSX.into()
}

pub const QUERY_JS: &str = r#"
    (function_declaration name: (identifier) @Function)
    (class_declaration name: (identifier) @Class)
    (method_definition name: (property_identifier) @Method)
    (call_expression function: (identifier) @Call)
    (call_expression function: (member_expression property: (property_identifier) @Call))
"#;

pub const QUERY_TS: &str = r#"
    (function_declaration name: (identifier) @Function)
    (class_declaration name: (type_identifier) @Class)
    (method_definition name: (property_identifier) @Method)
    (call_expression function: (identifier) @Call)
    (call_expression function: (member_expression property: (property_identifier) @Call))
"#;
