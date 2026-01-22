use tree_sitter::Language;

pub fn get_language() -> Language {
    tree_sitter_python::LANGUAGE.into()
}

pub const QUERY: &str = r#"
(function_definition 
    name: (identifier) @Function 
    body: (block (expression_statement (string) @doc)?))
(decorator 
    (call 
        function: (attribute attribute: (identifier) @route_method)
        arguments: (argument_list (string) @route_path)
    )
    (#match? @route_method "^(get|post|put|delete)$")
)
(call function: (identifier) @Call)
"#;