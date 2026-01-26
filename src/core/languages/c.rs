use tree_sitter::Language;

pub fn language() -> Language {
    tree_sitter_c::LANGUAGE.into()
}

pub const QUERY: &str = r#"
    (function_definition declarator: (function_declarator declarator: (identifier) @Function))
    (struct_specifier name: (type_identifier) @Struct)
    (call_expression function: (identifier) @Call)
"#;
