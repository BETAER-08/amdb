use std::path::Path;
use tree_sitter::{Language, Node};

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

pub trait SymbolEnricher {
    fn is_public(&self, node: Node, src: &str) -> bool;
    fn signature(&self, node: Node, src: &str) -> Option<String>;
}

pub fn signature_before_body(node: Node, src: &str) -> Option<String> {
    let end = node
        .child_by_field_name("body")
        .map(|b| b.start_byte())
        .unwrap_or_else(|| node.end_byte());

    let text = src.get(node.start_byte()..end)?.trim_end();
    if text.is_empty() {
        None
    } else {
        Some(text.to_string())
    }
}

pub fn has_child_of_kind(node: Node, kind: &str) -> bool {
    let mut cursor = node.walk();
    let found = node.children(&mut cursor).any(|c| c.kind() == kind);
    found
}

pub struct DefaultEnricher;

impl SymbolEnricher for DefaultEnricher {
    fn is_public(&self, _node: Node, _src: &str) -> bool {
        true
    }

    fn signature(&self, _node: Node, _src: &str) -> Option<String> {
        None
    }
}

static DEFAULT_ENRICHER: DefaultEnricher = DefaultEnricher;

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

    pub fn enricher(&self) -> &'static dyn SymbolEnricher {
        match self {
            Self::Rust => &rust::ENRICHER,
            Self::Python => &python::ENRICHER,
            Self::TypeScript | Self::Tsx => &javascript::TS_ENRICHER,
            _ => &DEFAULT_ENRICHER,
        }
    }

    pub fn get_query(&self) -> &'static str {
        match self {
            Self::Rust => rust::QUERY,
            Self::Python => python::QUERY,
            Self::JavaScript => javascript::QUERY_JS,
            Self::TypeScript | Self::Tsx => javascript::QUERY_TS,
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
