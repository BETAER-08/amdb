use tree_sitter::Language;

pub fn get_language() -> Language {
    tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()
}

pub const QUERY: &str = r#"
(function_declaration name: (identifier) @Function)
(class_declaration name: (type_identifier) @Class)
(interface_declaration name: (type_identifier) @Interface)
(method_definition name: (property_identifier) @Method)
(type_alias_declaration name: (type_identifier) @Type)
(call_expression function: (_) @Call) 
"#;