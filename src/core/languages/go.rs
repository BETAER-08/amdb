use tree_sitter::Language;

pub fn language() -> Language {
    tree_sitter_go::LANGUAGE.into()
}

pub const QUERY: &str = r#"
(function_declaration name: (identifier) @name) @function
(method_declaration name: (field_identifier) @name) @method
(type_declaration (type_specifier name: (type_identifier) @name)) @struct
"#;