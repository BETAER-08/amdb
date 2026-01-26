use tree_sitter::Language;

pub fn language() -> Language {
    tree_sitter_toml::LANGUAGE.into()
}

pub const QUERY: &str = r#"
    (pair key: (bare_key) @Key)
    (pair key: (quoted_key) @Key)
    (pair key: (dotted_key) @Key)
    (table header: (table_header) @Table)
"#;
