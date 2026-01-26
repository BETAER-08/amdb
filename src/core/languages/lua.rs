use tree_sitter::Language;

pub fn language() -> Language {
    tree_sitter_lua::LANGUAGE.into()
}

pub const QUERY: &str = r#"
    (function_definition name: (function_name) @Function)
    (function_call name: (identifier) @Call)
"#;
