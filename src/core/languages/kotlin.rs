use tree_sitter::Language;

pub fn language() -> Language {
    let lang = tree_sitter_kotlin::language();
    unsafe { std::mem::transmute(lang) }
}

pub const QUERY: &str = r#"
(class_declaration name: (simple_identifier) @name) @class
(object_declaration name: (simple_identifier) @name) @class
(function_declaration name: (simple_identifier) @name) @function
(interface_declaration name: (simple_identifier) @name) @interface
"#;