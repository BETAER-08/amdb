use tree_sitter::Language;

pub fn language() -> Language {
    tree_sitter_c_sharp::LANGUAGE.into()
}

pub const QUERY: &str = r#"
    (method_declaration name: (identifier) @Method)
    (class_declaration name: (identifier) @Class)
    (interface_declaration name: (identifier) @Interface)
    (invocation_expression function: (identifier) @Call) 
"#;
