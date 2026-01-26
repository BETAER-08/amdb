use tree_sitter::Language;

pub fn language() -> Language {
    tree_sitter_php::LANGUAGE_PHP.into()
}

pub const QUERY: &str = r#"
    (function_definition name: (name) @Function)
    (method_declaration name: (name) @Function)
    (class_declaration name: (name) @Class)
    (member_call_expression name: (name) @Call)
    (function_call_expression function: (qualified_name (name)) @Call)
"#;
