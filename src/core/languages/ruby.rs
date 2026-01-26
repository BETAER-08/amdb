use tree_sitter::Language;

pub fn language() -> Language {
    tree_sitter_ruby::LANGUAGE.into()
}

pub const QUERY: &str = r#"
    (method name: (identifier) @Function)
    (class name: (constant) @Class)
    (call method: (identifier) @Call)
"#;
