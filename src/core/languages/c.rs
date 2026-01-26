use tree_sitter::Language;

pub fn language() -> Language {
    tree_sitter_c::LANGUAGE.into()
}

pub const QUERY: &str = r#"
(function_definition declarator: (function_declarator declarator: (identifier) @name)) @function
(struct_specifier name: (type_identifier) @name) @struct
(enum_specifier name: (type_identifier) @name) @enum
"#;