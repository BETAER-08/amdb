use tree_sitter::Language;

pub fn language() -> Language {
    tree_sitter_bash::LANGUAGE.into()
}

pub const QUERY: &str = r#"
    (function_definition name: (word) @Function)
    (variable_assignment name: (variable_name) @Variable)
    (command_name) @Call
"#;
