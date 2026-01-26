use tree_sitter::Language;

pub fn language() -> Language {
    tree_sitter_java::LANGUAGE.into()
}

pub const QUERY: &str = r#"
    (method_declaration name: (identifier) @Function)
    (class_declaration name: (identifier) @Class)
    (method_invocation name: (identifier) @Call)
"#;
