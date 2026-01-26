use std::path::Path;
use tree_sitter::Language;

// 하위 모듈들 등록
pub mod rust;
pub mod python;
pub mod javascript;

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
            "js" | "jsx" | "mjs" => Some(Self::JavaScript),
            "ts" => Some(Self::TypeScript),
            "tsx" => Some(Self::Tsx),
            _ => None,
        }
    }

    pub fn get_language(&self) -> Language {
        match self {
            Self::Rust => rust::language(),
            Self::Python => python::language(),
            Self::JavaScript => javascript::language_js(),
            Self::TypeScript => javascript::language_ts(),
            Self::Tsx => javascript::language_tsx(),
        }
    }

    pub fn get_query(&self) -> &'static str {
        match self {
            Self::Rust => rust::QUERY,
            Self::Python => python::QUERY,
            Self::JavaScript | Self::TypeScript | Self::Tsx => javascript::QUERY,
        }
    }
}