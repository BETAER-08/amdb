use tree_sitter::Language;

pub fn language() -> Language {
    tree_sitter_php::language_php()
}

pub const QUERY: &str = r#"
(class_declaration name: (name) @name) @class
(method_declaration name: (name) @name) @method
(function_definition name: (name) @name) @function
(trait_declaration name: (name) @name) @trait
"#;