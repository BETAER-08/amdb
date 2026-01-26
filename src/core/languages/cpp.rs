use tree_sitter::Language;

pub fn language() -> Language {
    tree_sitter_cpp::LANGUAGE.into()
}

pub const QUERY: &str = r#"
    (function_definition declarator: (function_declarator declarator: (identifier) @Function))
    (class_specifier name: (type_identifier) @Class)
    (struct_specifier name: (type_identifier) @Struct)
    (call_expression function: (identifier) @Call)
"#;
