use std::path::Path;
use tree_sitter::Language;

pub mod rust;
pub mod python;

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
            Self::Rust => rust::get_language(),
            Self::Python => python::get_language(),
        }
    }

    pub fn get_query(&self) -> &'static str {
        match self {
            Self::Rust => rust::QUERY,
            Self::Python => python::QUERY,
        }
    }
}