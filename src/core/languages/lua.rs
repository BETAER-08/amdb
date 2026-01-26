use tree_sitter::Language;

pub fn language() -> Language {
    tree_sitter_lua::LANGUAGE.into()
}

pub const QUERY: &str = r#"
(function_declaration name: [(identifier) (dot_index_expression)] @name) @function
"#;