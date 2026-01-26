use tree_sitter::Language;

pub fn language() -> Language {
    tree_sitter_swift::LANGUAGE.into()
}

pub const QUERY: &str = r#"
    (function_declaration name: (simple_identifier) @Function)
    (class_declaration name: (type_identifier) @Class)
    (call_expression function: (simple_identifier) @Call)
"#;
