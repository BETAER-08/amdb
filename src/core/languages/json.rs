use tree_sitter::Language;

pub fn language() -> Language {
    tree_sitter_json::LANGUAGE.into()
}

pub const QUERY: &str = r#"
    (pair key: (string) @Key)
"#;
