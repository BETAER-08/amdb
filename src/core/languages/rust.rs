use tree_sitter::Language;

pub fn get_language() -> Language {
    tree_sitter_rust::LANGUAGE.into()
}

pub const QUERY: &str = r#"
(line_comment) @doc
(function_item 
    visibility_modifier: (visibility_modifier) @pub 
    name: (identifier) @Function 
    parameters: (parameters) @sig)
(call_expression function: (_) @Call)
"#;