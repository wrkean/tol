use crate::lexer::token_kind::TokenKind;

pub struct Token {
    lexeme: String,
    kind: TokenKind,
    line: usize,
    column: usize,
}

impl Token {
    pub fn new(lexeme: &str, kind: TokenKind, line: usize, column: usize) -> Self {
        Self {
            lexeme: lexeme.to_string(),
            kind,
            line,
            column,
        }
    }

    pub fn lexeme(&self) -> &str {
        &self.lexeme
    }

    pub fn kind(&self) -> &TokenKind {
        &self.kind
    }

    pub fn line(&self) -> usize {
        self.line
    }

    pub fn column(&self) -> usize {
        self.column
    }
}
