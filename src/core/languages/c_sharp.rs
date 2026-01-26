use tree_sitter::Language;

pub fn language() -> Language {
    tree_sitter_c_sharp::language()
}

pub const QUERY: &str = r#"
(class_declaration name: (identifier) @name) @class
(interface_declaration name: (identifier) @name) @interface
(method_declaration name: (identifier) @name) @method
(struct_declaration name: (identifier) @name) @struct
(enum_declaration name: (identifier) @name) @enum
"#;