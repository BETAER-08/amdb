use tree_sitter::Language;

extern "C" {
    fn tree_sitter_lua() -> Language;
}

pub fn language() -> Language {
    unsafe { tree_sitter_lua() }
}

pub const QUERY: &str = r#"
    (function_definition name: (function_name) @Function)
    (function_call name: (identifier) @Call)
"#;
