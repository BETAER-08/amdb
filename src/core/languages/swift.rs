use tree_sitter::Language;

pub fn language() -> Language {
    tree_sitter_swift::LANGUAGE.into()
}

pub const QUERY: &str = r#"
(class_declaration name: (type_identifier) @name) @class
(struct_declaration name: (type_identifier) @name) @struct
(protocol_declaration name: (type_identifier) @name) @interface
(function_declaration name: (simple_identifier) @name) @function
"#;