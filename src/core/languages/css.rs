use tree_sitter::Language;

pub fn language() -> Language {
    tree_sitter_css::LANGUAGE.into()
}

pub const QUERY: &str = r#"
    (class_selector (class_name) @Class)
    (id_selector (id_name) @Id)
    (tag_name) @Tag
    (property_name) @Property
"#;
