use tree_sitter::Language;

pub fn language() -> Language {
    tree_sitter_ruby::LANGUAGE.into()
}

pub const QUERY: &str = r#"
    (method name: (identifier) @Method)
    (class name: (constant) @Class)
    (module name: (constant) @Module)
    (call method: (identifier) @Call)
"#;
