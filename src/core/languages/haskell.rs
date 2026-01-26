use tree_sitter::Language;

pub fn language() -> Language {
    tree_sitter_haskell::LANGUAGE.into()
}

pub const QUERY: &str = r#"
    (function name: (variable) @Function)
    (type name: (type_name) @Class)
    (exp_apply function: (variable) @Call)
"#;
