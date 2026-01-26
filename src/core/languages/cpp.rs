use tree_sitter::Language;

pub fn language() -> Language {
    tree_sitter_cpp::LANGUAGE.into()
}

pub const QUERY: &str = r#"
(function_definition declarator: (function_declarator declarator: (identifier) @name)) @function
(class_specifier name: (type_identifier) @name) @class
(struct_specifier name: (type_identifier) @name) @struct
(template_declaration (class_specifier name: (type_identifier) @name)) @class
"#;