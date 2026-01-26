use tree_sitter::Language;

pub fn language() -> Language {
    tree_sitter_html::LANGUAGE.into()
}

pub const QUERY: &str = r#"
    (element_name) @Tag
    (attribute_name) @Attribute
    (id) @Id
    (class_name) @Class
"#;
