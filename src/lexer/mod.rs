use std::{collections::HashMap, iter::Peekable, str::Chars};

use crate::{
    error::{CompilerError, ErrorKind},
    lexer::{token::Token, token_kind::TokenKind},
};

pub mod token;
pub mod token_kind;

pub struct Lexer<'a> {
    source: &'a str,
    source_path: &'a str,
    chars: Peekable<Chars<'a>>,
    keywords: HashMap<&'static str, TokenKind>,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
    column: usize,
    start_column: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str, source_path: &'a str) -> Self {
        let keywords = HashMap::from([
            ("par", TokenKind::Par),
            ("ang", TokenKind::Ang),
            ("maiba", TokenKind::Maiba),
        ]);
        Self {
            source,
            source_path,
            chars: source.chars().peekable(),
            keywords,
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
            column: 1,
            start_column: 1,
        }
    }

    pub fn lex(&mut self) -> &Vec<Token> {
        while !self.is_at_end() {
            self.start = self.current;
            self.start_column = self.column;

            if let Err(e) = self.next_token() {
                e.display(self.source_path);
            }
        }

        if !matches!(self.tokens.last().map(|t| t.kind()), Some(TokenKind::Eof)) {
            self.add_token(TokenKind::Eof, Some("Eof"));
        }

        &self.tokens
    }

    fn next_token(&mut self) -> Result<(), CompilerError> {
        let ch = match self.advance() {
            Some(c) => c,
            None => {
                self.add_token(TokenKind::Eof, Some("EOF"));
                return Ok(());
            }
        };

        match ch {
            '(' => self.add_token(TokenKind::LeftParen, None),
            ')' => self.add_token(TokenKind::RightParen, None),
            '{' => self.add_token(TokenKind::LeftBrace, None),
            '}' => self.add_token(TokenKind::RightBrace, None),
            ':' => self.add_token(TokenKind::Colon, None),
            ';' => self.add_token(TokenKind::SemiColon, None),
            ',' => self.add_token(TokenKind::Comma, None),
            '+' => {
                if self.match_char('=') {
                    self.add_token(TokenKind::PlusEqual, None);
                } else {
                    self.add_token(TokenKind::Plus, None);
                }
            }
            '-' => {
                if self.match_char('>') {
                    self.add_token(TokenKind::ThinArrow, None);
                } else if self.match_char('=') {
                    self.add_token(TokenKind::MinusEqual, None);
                } else {
                    self.add_token(TokenKind::Minus, None);
                }
            }
            '*' => {
                if self.match_char('=') {
                    self.add_token(TokenKind::StarEqual, None);
                } else {
                    self.add_token(TokenKind::Star, None);
                }
            }
            '/' => {
                if self.match_char('=') {
                    self.add_token(TokenKind::SlashEqual, None);
                } else {
                    self.add_token(TokenKind::Slash, None);
                }
            }
            '%' => {
                if self.match_char('=') {
                    self.add_token(TokenKind::PercentEqual, None);
                } else {
                    self.add_token(TokenKind::Percent, None);
                }
            }
            '=' => {
                if self.match_char('=') {
                    self.add_token(TokenKind::EqualEqual, None);
                } else {
                    self.add_token(TokenKind::Equal, None);
                }
            }
            '!' => {
                if self.match_char('=') {
                    self.add_token(TokenKind::BangEqual, None);
                } else {
                    return Err(CompilerError::new(
                        "Hindi suportadong token '!'",
                        ErrorKind::Error,
                        self.line,
                        self.start_column,
                    )
                    .with_help("Palitan mo ito ng `di`"));
                }
            }
            '>' => {
                if self.match_char('=') {
                    self.add_token(TokenKind::GreaterEqual, None);
                } else {
                    self.add_token(TokenKind::Greater, None);
                }
            }
            '<' => {
                if self.match_char('=') {
                    self.add_token(TokenKind::LesserEqual, None);
                } else {
                    self.add_token(TokenKind::Lesser, None);
                }
            }
            c if c.is_whitespace() => {
                self.skip_whitespace();
            }
            '\n' => {
                self.line += 1;
                self.column = 1;
            }
            _ => {
                if self.is_identifier_start(ch) {
                    self.lex_identifier();
                } else if ch.is_ascii_digit() {
                    self.lex_number();
                } else {
                    return Err(CompilerError::new(
                        &format!("Hindi valid na karakter: `{ch}`"),
                        ErrorKind::Error,
                        self.line,
                        self.start_column,
                    )
                    .with_note("Siguro ito ay hindi parte ng sintax ng Tol")
                    .with_help("Subukan mo itong tanggalin"));
                }
            }
        }

        Ok(())
    }

    fn lex_identifier(&mut self) {
        while let Some(&ch) = self.peek() {
            if self.is_identifier_continue(ch) {
                self.advance();
            } else {
                break;
            }
        }

        let lexeme = &self.source[self.start..self.current];

        match self.keywords.get(lexeme) {
            Some(keyword_kind) => self.add_token(keyword_kind.clone(), Some(lexeme)),
            None => self.add_token(TokenKind::Identifier, Some(lexeme)),
        }
    }

    fn lex_number(&mut self) {
        let mut is_float = false;
        while let Some(&ch) = self.peek() {
            if ch.is_ascii_digit() || ch == '_' {
                self.advance();
            } else {
                break;
            }
        }

        if let Some(&'.') = self.peek() {
            is_float = true;
            self.advance();

            while let Some(&ch) = self.peek() {
                if ch.is_ascii_digit() || ch == '_' {
                    self.advance();
                } else {
                    break;
                }
            }
        }

        let with_underscores = &self.source[self.start..self.current];

        let without_underscores: String = with_underscores.chars().filter(|&c| c != '_').collect();

        let kind = if is_float {
            TokenKind::FloatLit
        } else {
            TokenKind::IntLit
        };

        self.add_token(kind, Some(&without_underscores));
    }

    fn add_token(&mut self, kind: TokenKind, literal: Option<&str>) {
        let lexeme = match literal {
            Some(lxm) => lxm,
            None => &self.source[self.start..self.current],
        };

        self.tokens
            .push(Token::new(lexeme, kind, self.line, self.start_column));
    }

    /// Check if a character can start an identifier (UAX #31 compliant)
    fn is_identifier_start(&self, ch: char) -> bool {
        unicode_ident::is_xid_start(ch)
    }

    /// Check if a character can continue an identifier (UAX #31 compliant)
    fn is_identifier_continue(&self, ch: char) -> bool {
        unicode_ident::is_xid_continue(ch)
    }

    fn is_ascii_identifier_start(&self, ch: char) -> bool {
        ch.is_ascii_alphabetic() || ch == '_'
    }

    fn is_ascii_identifier_continue(&self, ch: char) -> bool {
        ch.is_ascii_alphanumeric() || ch == '_'
    }

    fn skip_whitespace(&mut self) {
        while let Some(&ch) = self.peek() {
            // Avoiding newlines prevents issues with
            // incorrect line number and oclumn number given
            // by errors
            if ch.is_whitespace() && ch != '\n' {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn peek(&mut self) -> Option<&char> {
        self.chars.peek()
    }

    fn advance(&mut self) -> Option<char> {
        if let Some(ch) = self.chars.next() {
            self.current += ch.len_utf8();
            self.column += 1;
            Some(ch)
        } else {
            None
        }
    }

    fn match_char(&mut self, expected: char) -> bool {
        if let Some(ch) = self.peek() {
            if *ch == expected {
                self.advance();
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    fn is_at_end(&mut self) -> bool {
        self.peek().is_none()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn lex(source: &str) -> Vec<Token> {
        let mut lexer = Lexer::new(source, "test");
        lexer.lex().clone()
    }

    #[test]
    fn test_single_char_tokens() {
        let tokens: Vec<TokenKind> = lex("( ) { } : ; + - * / % = > <")
            .iter()
            .map(|t| t.kind().clone())
            .collect();

        assert_eq!(tokens.len(), 15);
        assert_eq!(
            tokens,
            vec![
                TokenKind::LeftParen,
                TokenKind::RightParen,
                TokenKind::LeftBrace,
                TokenKind::RightBrace,
                TokenKind::Colon,
                TokenKind::SemiColon,
                TokenKind::Plus,
                TokenKind::Minus,
                TokenKind::Star,
                TokenKind::Slash,
                TokenKind::Percent,
                TokenKind::Equal,
                TokenKind::Greater,
                TokenKind::Lesser,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_combined_tokens() {
        let tokens: Vec<TokenKind> = lex("+= -= *= /= %= == != >= <= ->")
            .iter()
            .map(|t| t.kind().clone())
            .collect();

        assert_eq!(tokens.len(), 11);
        assert_eq!(
            tokens,
            vec![
                TokenKind::PlusEqual,
                TokenKind::MinusEqual,
                TokenKind::StarEqual,
                TokenKind::SlashEqual,
                TokenKind::PercentEqual,
                TokenKind::EqualEqual,
                TokenKind::BangEqual,
                TokenKind::GreaterEqual,
                TokenKind::LesserEqual,
                TokenKind::ThinArrow,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_identifiers_and_keywords() {
        let tokens: Vec<TokenKind> = lex("par ang maiba myVar _anotherVar var123")
            .iter()
            .map(|t| t.kind().clone())
            .collect();

        assert_eq!(tokens.len(), 7);
        assert_eq!(
            tokens,
            vec![
                TokenKind::Par,
                TokenKind::Ang,
                TokenKind::Maiba,
                TokenKind::Identifier,
                TokenKind::Identifier,
                TokenKind::Identifier,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_floats_and_ints() {
        let tokens: Vec<TokenKind> = lex("123 456_789 3.14 0.001 1_000.000_1")
            .iter()
            .map(|t| t.kind().clone())
            .collect();

        assert_eq!(tokens.len(), 6);
        assert_eq!(
            tokens,
            vec![
                TokenKind::IntLit,
                TokenKind::IntLit,
                TokenKind::FloatLit,
                TokenKind::FloatLit,
                TokenKind::FloatLit,
                TokenKind::Eof,
            ]
        );

        let lexemes: Vec<String> = lex("123 456_789 3.14 0.001 1_000.000_1")
            .iter()
            .map(|t| t.lexeme().to_string())
            .collect();

        assert_eq!(
            lexemes,
            vec!["123", "456789", "3.14", "0.001", "1000.0001", "Eof"]
        );
    }

    #[test]
    fn test_invalid_tokens() {
        let tokens: Vec<TokenKind> = lex("! $ @ # ^ &")
            .iter()
            .map(|t| t.kind().clone())
            .collect();

        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens, vec![TokenKind::Eof]);
    }
}
