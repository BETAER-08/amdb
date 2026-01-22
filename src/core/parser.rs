use anyhow::Result;
use tree_sitter::{Parser, Query, QueryCursor};

// [핵심 수정] use -> pub use 로 변경하여 외부에서 이 타입을 쓸 수 있게 공개합니다.
pub use super::languages::SupportedLanguage;

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
        
        // 이제 언어별로 분리된 쿼리 문자열을 가져옵니다
        let query_str = self.language.get_query();
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