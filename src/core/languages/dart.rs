use tree_sitter::Language;

pub fn language() -> Language {
    tree_sitter_dart::LANGUAGE.into()
}

pub const QUERY: &str = r#"
    (class_definition name: (identifier) @Class)
    (function_signature name: (identifier) @Function)
    (method_signature name: (identifier) @Method)
    (variable_declaration name: (identifier) @Variable)
    (call_expression function: (identifier) @Call)
"#;
