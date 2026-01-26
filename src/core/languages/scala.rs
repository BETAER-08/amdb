use tree_sitter::Language;

pub fn language() -> Language {
    tree_sitter_scala::language()
}

pub const QUERY: &str = r#"
(class_definition name: (identifier) @name) @class
(object_definition name: (identifier) @name) @class
(trait_definition name: (identifier) @name) @interface
(function_definition name: (identifier) @name) @function
"#;