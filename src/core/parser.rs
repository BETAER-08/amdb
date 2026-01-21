use anyhow::Result;
use std::path::Path;
use tree_sitter::{Parser, Language, Query, QueryCursor};

#[derive(Clone, Copy)]
pub enum SupportedLanguage {
    Rust,
    Python,
}

impl SupportedLanguage {
    pub fn from_path(path: &Path) -> Option<Self> {
        match path.extension()?.to_str()? {
            "rs" => Some(Self::Rust),
            "py" => Some(Self::Python),
            _ => None,
        }
    }

    pub fn get_language(&self) -> Language {
        match self {
            Self::Rust => tree_sitter_rust::language(),
            Self::Python => tree_sitter_python::language(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CodeSymbol {
    pub kind: String,
    pub name: String,
    pub line: usize,
}

pub struct CodeParser {
    parser: Parser,
    language: SupportedLanguage,
}

impl CodeParser {
    pub fn new(lang: SupportedLanguage) -> Result<Self> {
        let mut parser = Parser::new();
        parser.set_language(lang.get_language())?;
        Ok(Self { parser, language: lang })
    }

    pub fn parse_symbols(&mut self, code: &str) -> Result<Vec<CodeSymbol>> {
        let tree = self.parser.parse(code, None)
            .ok_or_else(|| anyhow::anyhow!("Parsing failed"))?;
        
        let query_str = match self.language {
            SupportedLanguage::Rust => r#"
                (function_item name: (identifier) @Function)
                (struct_item name: (type_identifier) @Struct)
                (enum_item name: (type_identifier) @Enum)
                (mod_item name: (identifier) @Module)
            "#,
            SupportedLanguage::Python => r#"
                (function_definition name: (identifier) @Function)
                (class_definition name: (identifier) @Class)
            "#,
        };

        let query = Query::new(self.language.get_language(), query_str)?;
        let mut cursor = QueryCursor::new();
        let mut symbols = Vec::new();

        for m in cursor.matches(&query, tree.root_node(), code.as_bytes()) {
            for capture in m.captures {
                let kind = query.capture_names()[capture.index as usize].to_string();
                let name = capture.node.utf8_text(code.as_bytes())?.to_string();
                let line = capture.node.start_position().row + 1;

                symbols.push(CodeSymbol { kind, name, line });
            }
        }
        
        Ok(symbols)
    }
}