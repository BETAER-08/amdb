use tree_sitter::Language;

pub fn language() -> Language {
    tree_sitter_yaml::LANGUAGE.into()
}

pub const QUERY: &str = r#"
    (block_mapping_pair key: (flow_node) @Key)
    (block_mapping_pair key: (plain_scalar) @Key)
"#;
