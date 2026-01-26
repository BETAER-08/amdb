use tree_sitter::Language;

pub fn language() -> Language {
    tree_sitter_ruby::LANGUAGE.into()
}

pub const QUERY: &str = r#"
(class name: (constant) @name) @class
(module name: (constant) @name) @module
(method name: (identifier) @name) @method
"#;