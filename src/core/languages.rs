use std::path::Path;
use tree_sitter::Language;

#[derive(Debug, Clone, Copy)]
pub enum SupportedLanguage {
    Rust,
    Python,
    JavaScript,
    TypeScript,
    Tsx, 
}

impl SupportedLanguage {
    pub fn from_path(path: &Path) -> Option<Self> {
        match path.extension()?.to_str()? {
            "rs" => Some(Self::Rust),
            "py" => Some(Self::Python),
            "js" => Some(Self::JavaScript),
            "ts" => Some(Self::TypeScript),
            "tsx" => Some(Self::Tsx),
            _ => None,
        }
    }

    pub fn get_language(&self) -> Language {
        match self {
            Self::Rust => tree_sitter_rust::LANGUAGE.into(),
            Self::Python => tree_sitter_python::LANGUAGE.into(),
            Self::JavaScript => tree_sitter_javascript::LANGUAGE.into(),
            Self::TypeScript => tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
            Self::Tsx => tree_sitter_typescript::LANGUAGE_TSX.into(),
        }
    }

    pub fn get_query(&self) -> &'static str {
        match self {
            Self::Rust => r#"
                (function_item name: (identifier) @Function)
                (struct_item name: (type_identifier) @Struct)
                (impl_item type: (type_identifier) @Class)
                (call_expression function: (identifier) @Call)
                (call_expression function: (field_expression field: (identifier) @Call))
                (call_expression function: (scoped_identifier name: (identifier) @Call))
            "#,
            Self::Python => r#"
                (function_definition name: (identifier) @Function)
                (class_definition name: (identifier) @Class)
                (call function: (identifier) @Call)
                (call function: (attribute attribute: (identifier) @Call))
            "#,
            Self::JavaScript | Self::TypeScript | Self::Tsx => r#"
                (function_declaration name: (identifier) @Function)
                (class_declaration name: (identifier) @Class)
                (method_definition name: (property_identifier) @Method)
                (call_expression function: (identifier) @Call)
                (call_expression function: (member_expression property: (property_identifier) @Call))
            "#,
        }
    }
}