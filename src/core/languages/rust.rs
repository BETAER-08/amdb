use tree_sitter::Language;

pub fn get_language() -> Language {
    tree_sitter_rust::language()
}

pub const QUERY: &str = r#"
(function_item name: (identifier) @Function)
(struct_item name: (type_identifier) @Struct)
(enum_item name: (type_identifier) @Enum)
(mod_item name: (identifier) @Module)
"#;