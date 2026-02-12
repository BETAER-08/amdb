use tree_sitter::Language;

pub fn language() -> Language {
    tree_sitter_rust::LANGUAGE.into()
}

pub const QUERY: &str = r#"
    (function_item name: (identifier) @Function)
    (struct_item name: (type_identifier) @Struct)
    (impl_item type: (type_identifier) @Class)
    (call_expression function: [
        (identifier) @Call
        (field_expression field: (field_identifier) @Call)
        (scoped_identifier name: (identifier) @Call)
    ])
"#;
