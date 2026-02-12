use std::path::Path;
use tree_sitter::Language;

pub mod bash;
pub mod c;
pub mod cpp;
pub mod csharp;
pub mod css;
pub mod go;
pub mod html;
pub mod java;
pub mod javascript;
pub mod json;
pub mod php;
pub mod python;
pub mod ruby;
pub mod rust;

#[derive(Debug, Clone, Copy)]
pub enum SupportedLanguage {
    Rust,
    Python,
    JavaScript,
    TypeScript,
    Tsx,
    C,
    Cpp,
    Go,
    Java,
    CSharp,
    Ruby,
    Php,
    Html,
    Css,
    Json,
    Bash,
}

impl SupportedLanguage {
    pub fn from_path(path: &Path) -> Option<Self> {
        match path.extension()?.to_str()? {
            "rs" => Some(Self::Rust),
            "py" => Some(Self::Python),
            "js" | "jsx" | "mjs" => Some(Self::JavaScript),
            "ts" => Some(Self::TypeScript),
            "tsx" => Some(Self::Tsx),
            "c" | "h" => Some(Self::C),
            "cpp" | "hpp" | "cc" | "cxx" => Some(Self::Cpp),
            "go" => Some(Self::Go),
            "java" => Some(Self::Java),
            "cs" => Some(Self::CSharp),
            "rb" => Some(Self::Ruby),
            "php" => Some(Self::Php),
            "html" | "htm" => Some(Self::Html),
            "css" => Some(Self::Css),
            "json" => Some(Self::Json),
            "sh" | "bash" => Some(Self::Bash),
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
            Self::C => c::language(),
            Self::Cpp => cpp::language(),
            Self::Go => go::language(),
            Self::Java => java::language(),
            Self::CSharp => csharp::language(),
            Self::Ruby => ruby::language(),
            Self::Php => php::language(),
            Self::Html => html::language(),
            Self::Css => css::language(),
            Self::Json => json::language(),
            Self::Bash => bash::language(),
        }
    }

    pub fn get_query(&self) -> &'static str {
        match self {
            Self::Rust => rust::QUERY,
            Self::Python => python::QUERY,
            Self::JavaScript | Self::TypeScript | Self::Tsx => javascript::QUERY,
            Self::C => c::QUERY,
            Self::Cpp => cpp::QUERY,
            Self::Go => go::QUERY,
            Self::Java => java::QUERY,
            Self::CSharp => csharp::QUERY,
            Self::Ruby => ruby::QUERY,
            Self::Php => php::QUERY,
            Self::Html => html::QUERY,
            Self::Css => css::QUERY,
            Self::Json => json::QUERY,
            Self::Bash => bash::QUERY,
        }
    }
}
