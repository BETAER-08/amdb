use anyhow::Result;
use std::path::Path;
use tree_sitter::{Parser, Language};

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

pub struct CodeParser {
    parser: Parser,
}

impl CodeParser {
    pub fn new(lang: SupportedLanguage) -> Result<Self> {
        let mut parser = Parser::new();
        parser.set_language(lang.get_language())?;
        Ok(Self { parser })
    }

    pub fn parse(&mut self, code: &str) -> Result<String> {
        let tree = self.parser.parse(code, None)
            .ok_or_else(|| anyhow::anyhow!("Parsing failed"))?;
        Ok(tree.root_node().to_sexp())
    }
}