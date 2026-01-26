use tree_sitter::Language;

extern "C" {
    fn tree_sitter_kotlin() -> *const ();
}

pub fn language() -> Language {
    unsafe { Language::from_raw(tree_sitter_kotlin() as *const _) }
}

pub const QUERY: &str = r#"
(class_declaration name: (simple_identifier) @name) @class
(object_declaration name: (simple_identifier) @name) @class
(function_declaration name: (simple_identifier) @name) @function
(interface_declaration name: (simple_identifier) @name) @interface
"#;