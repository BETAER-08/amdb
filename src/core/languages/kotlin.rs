use tree_sitter::Language;

pub fn language() -> Language {
    tree_sitter_kotlin::language()
}

pub const QUERY: &str = r#"
(class_declaration name: (simple_identifier) @name) @class
(object_declaration name: (simple_identifier) @name) @class
(function_declaration name: (simple_identifier) @name) @function
(interface_declaration name: (simple_identifier) @name) @interface
"#;