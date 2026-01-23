use tree_sitter::Language;

pub fn language() -> Language {
    tree_sitter_python::LANGUAGE.into()
}

pub const QUERY: &str = r#"
    (function_definition
      name: (identifier) @Function
      body: (block . (expression_statement (string) @doc)?)
    )
    (class_definition
      name: (identifier) @Class
      body: (block . (expression_statement (string) @doc)?)
    )
    (call
      function: [
        (attribute object: (_) attribute: (identifier) @Call)
        (identifier) @Call
      ]
    )
"#;