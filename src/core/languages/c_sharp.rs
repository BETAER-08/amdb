use tree_sitter::Language;

pub fn language() -> Language {
    tree_sitter_c_sharp::LANGUAGE.into()
}

pub const QUERY: &str = r#"
    (method_declaration name: (identifier) @Function)
    (class_declaration name: (identifier) @Class)
    (struct_declaration name: (identifier) @Struct)
    (invocation_expression function: (identifier) @Call)
"#;
