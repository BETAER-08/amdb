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
pub mod swift;
pub mod kotlin;
pub mod scala;
pub mod haskell;
pub mod lua;

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
    Scala,
    Haskell,
    Lua,
}

impl SupportedLanguage {
    pub fn from_path(path: &Path) -> Option<Self> {
        let ext = path.extension()?.to_str()?;
        match ext {
            "rs" => Some(Self::Rust),
            "py" => Some(Self::Python),
            "js" | "jsx" | "mjs" => Some(Self::JavaScript),
            "ts" => Some(Self::TypeScript),
            "tsx" => Some(Self::Tsx),
            "go" => Some(Self::Go),
            "c" | "h" => Some(Self::C),
            "cpp" | "hpp" | "cc" | "cxx" => Some(Self::Cpp),
            "java" => Some(Self::Java),
            "cs" => Some(Self::CSharp),
            "php" => Some(Self::Php),
            "rb" => Some(Self::Ruby),
            "swift" => Some(Self::Swift),
            "kt" | "kts" => Some(Self::Kotlin),
            "scala" | "sc" => Some(Self::Scala),
            "hs" => Some(Self::Haskell),
            "lua" => Some(Self::Lua),
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
            Self::Go => go::language(),
            Self::C => c::language(),
            Self::Cpp => cpp::language(),
            Self::Java => java::language(),
            Self::CSharp => c_sharp::language(),
            Self::Php => php::language(),
            Self::Ruby => ruby::language(),
            Self::Swift => swift::language(),
            Self::Kotlin => kotlin::language(),
            Self::Scala => scala::language(),
            Self::Haskell => haskell::language(),
            Self::Lua => lua::language(),
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