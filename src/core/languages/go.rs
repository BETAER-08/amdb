use tree_sitter::Language;

pub fn language() -> Language {
    tree_sitter_go::LANGUAGE.into()
}

pub const QUERY: &str = r#"
    (function_declaration name: (identifier) @Function)
    (method_declaration name: (field_identifier) @Method)
    (type_declaration (type_specifier name: (type_identifier) @Struct))
    (call_expression function: (identifier) @Call)
"#;
