use tree_sitter::Language;

pub fn get_language() -> Language {
    tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()
}

pub const QUERY: &str = r#"
(comment) @doc
(function_declaration 
    name: (identifier) @Function 
    parameters: (formal_parameters) @sig)
(export_statement) @pub
(call_expression 
    function: (member_expression property: (property_identifier) @route_method)
    arguments: (arguments (string (string_fragment) @route_path))
    (#match? @route_method "^(get|post|put|delete|patch)$")
)
(call_expression function: (identifier) @Call)
"#;