use tree_sitter::Language;

pub fn language_js() -> Language {
    tree_sitter_javascript::LANGUAGE.into()
}

pub fn language_ts() -> Language {
    tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()
}

pub fn language_tsx() -> Language {
    tree_sitter_typescript::LANGUAGE_TSX.into()
}

pub const QUERY: &str = r#"
    (function_declaration name: (identifier) @Function)
    (class_declaration name: (identifier) @Class)
    (method_definition name: (property_identifier) @Method)
    (call_expression function: (identifier) @Call)
    (call_expression function: (member_expression property: (property_identifier) @Call))
"#;