use tree_sitter::Language;

pub fn get_language() -> Language {
    tree_sitter_python::LANGUAGE.into()
}

pub const QUERY: &str = r#"
(function_definition name: (identifier) @Function)
(class_definition name: (identifier) @Class)
(call function: (_) @Call) 
"#;