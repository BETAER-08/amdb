use tree_sitter::Language;

extern "C" {
    fn tree_sitter_kotlin() -> Language;
}

pub fn language() -> Language {
    unsafe { tree_sitter_kotlin() }
}

pub const QUERY: &str = r#"
    (function_declaration name: (simple_identifier) @Function)
    (class_declaration name: (simple_identifier) @Class)
    (call_expression expression: (simple_identifier) @Call)
"#;
