use tree_sitter::Language;

pub fn language() -> Language {
    tree_sitter_java::LANGUAGE.into()
}

pub const QUERY: &str = r#"
(class_declaration name: (identifier) @name) @class
(interface_declaration name: (identifier) @name) @interface
(method_declaration name: (identifier) @name) @method
(enum_declaration name: (identifier) @name) @enum
"#;