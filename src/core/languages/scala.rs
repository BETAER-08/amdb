use tree_sitter::Language;

pub fn language() -> Language {
    tree_sitter_scala::LANGUAGE.into()
}

pub const QUERY: &str = r#"
    (function_definition name: (identifier) @Function)
    (class_definition name: (identifier) @Class)
    (call_expression function: (identifier) @Call)
"#;
