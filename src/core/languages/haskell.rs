use tree_sitter::Language;

pub fn language() -> Language {
    tree_sitter_haskell::LANGUAGE.into()
}

pub const QUERY: &str = r#"
(function name: (variable) @name) @function
(data_type name: (type) @name) @struct
(newtype name: (type) @name) @struct
(class name: (type) @name) @interface
"#;