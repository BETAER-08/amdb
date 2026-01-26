use std::path::Path;
use tree_sitter::Language;

pub mod rust;
pub mod python;
pub mod javascript;
pub mod go;
pub mod c;
pub mod cpp;
pub mod java;
pub mod c_sharp;
pub mod php;
pub mod ruby;
// pub mod swift;
// pub mod kotlin;
pub mod scala;
pub mod haskell;
// pub mod lua;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SupportedLanguage {
    Rust,
    Python,
    JavaScript,
    TypeScript,
    Tsx,
    Go,
    C,
    Cpp,
    Java,
    CSharp,
    Php,
    Ruby,
    Swift,
    Kotlin,
    Lua,
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
            "cs" => Some(Self::CSharp),
            "go" => Some(Self::Go),
            "java" => Some(Self::Java),
            "kt" | "kts" => Some(Self::Kotlin),
             "lua" => Some(Self::Lua),
            "php" => Some(Self::Php),
            "rb" => Some(Self::Ruby),
            "scala" | "sc" => Some(Self::Scala),
            "swift" => Some(Self::Swift),
            "hs" | "lhs" => Some(Self::Haskell),
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
            Self::CSharp => c_sharp::language(),
            Self::Go => go::language(),
            Self::Java => java::language(),
            Self::Kotlin => kotlin::language(),
            Self::Lua => lua::language(),
            Self::Php => php::language(),
            Self::Ruby => ruby::language(),
            Self::Scala => scala::language(),
            Self::Swift => swift::language(),
            Self::Haskell => haskell::language(),
        }
    }

    pub fn get_query(&self) -> &'static str {
        match self {
            Self::Rust => rust::QUERY,
            Self::Python => python::QUERY,
            Self::JavaScript | Self::TypeScript | Self::Tsx => javascript::QUERY,
            Self::Go => go::QUERY,
            Self::C => c::QUERY,
            Self::Cpp => cpp::QUERY,
            Self::Java => java::QUERY,
            Self::CSharp => c_sharp::QUERY,
            Self::Php => php::QUERY,
            Self::Ruby => ruby::QUERY,
            Self::Swift => swift::QUERY,
            Self::Kotlin => kotlin::QUERY,
            Self::Scala => scala::QUERY,
            Self::Haskell => haskell::QUERY,
            Self::Lua => lua::QUERY,
        }
    }
}