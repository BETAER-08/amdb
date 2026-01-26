use tree_sitter::Language;

pub fn language() -> Language {
    tree_sitter_kotlin::LANGUAGE.into()
}

pub const QUERY: &str = r#"
    (function_declaration name: (simple_identifier) @Function)
    (class_declaration name: (simple_identifier) @Class)
    (call_expression expression: (simple_identifier) @Call)
"#;
