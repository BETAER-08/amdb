use tree_sitter::Language;

pub fn language() -> Language {
    tree_sitter_go::LANGUAGE.into()
}

pub const QUERY: &str = r#"
    (function_declaration name: (identifier) @Function)
    (method_declaration name: (field_identifier) @Function)
    (type_specifier name: (type_identifier) @Struct type: (struct_type))
    (call_expression function: (identifier) @Call)
    (call_expression function: (selector_expression field: (field_identifier) @Call))
"#;
