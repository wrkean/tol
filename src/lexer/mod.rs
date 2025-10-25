use std::collections::HashMap;

use crate::{
    error::{CompilerError, ErrorKind},
    lexer::{token::Token, token_kind::TokenKind},
    parser::module::Module,
};

pub mod token;
pub mod token_kind;

enum StringType {
    Byte,
    Normal,
}

pub struct Lexer<'a> {
    parent_module: &'a mut Module,
    keywords: HashMap<String, TokenKind>,
    start: usize,
    current: usize,
    line: usize,
    column: usize,
    start_column: usize,
    has_error: bool,
}

impl<'a> Lexer<'a> {
    pub fn new(parent_module: &'a mut Module) -> Self {
        let keywords = HashMap::from([
            ("paraan".to_string(), TokenKind::Paraan),
            ("ang".to_string(), TokenKind::Ang),
            ("maiba".to_string(), TokenKind::Maiba),
            ("ibalik".to_string(), TokenKind::Ibalik),
            ("bagay".to_string(), TokenKind::Bagay),
            ("itupad".to_string(), TokenKind::Itupad),
            ("kung".to_string(), TokenKind::Kung),
            ("kungdi".to_string(), TokenKind::KungDi),
            ("kungwala".to_string(), TokenKind::KungWala),
            ("sa".to_string(), TokenKind::Sa),
        ]);

        Self {
            parent_module,
            keywords,
            start: 0,
            current: 0,
            line: 1,
            column: 1,
            start_column: 1,
            has_error: false,
        }
    }

    pub fn lex(&mut self) {
        while !self.is_at_end() {
            self.start = self.current;
            self.start_column = self.column;

            if let Err(e) = self.next_token() {
                e.display(&self.parent_module.source_path);
                self.has_error = true;
            }
        }

        if !matches!(
            self.parent_module.tokens.last().map(|t| t.kind()),
            Some(TokenKind::Eof)
        ) {
            self.add_token(TokenKind::Eof, Some("Eof"));
        }
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
            '[' => self.add_token(TokenKind::LeftBracket, None),
            ']' => self.add_token(TokenKind::RightBracket, None),
            ';' => self.add_token(TokenKind::SemiColon, None),
            ',' => self.add_token(TokenKind::Comma, None),
            '@' => self.add_token(TokenKind::At, None),
            '&' => self.add_token(TokenKind::Amper, None),
            '.' => {
                if self.match_char('.') {
                    if self.match_char('=') {
                        self.add_token(TokenKind::DotDotEqual, None);
                    } else {
                        self.add_token(TokenKind::DotDot, None);
                    }
                } else {
                    self.add_token(TokenKind::Dot, None);
                }
            }
            ':' => {
                if self.match_char(':') {
                    self.add_token(TokenKind::ColonColon, None);
                } else {
                    self.add_token(TokenKind::Colon, None);
                }
            }
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
                } else if self.match_char('/') {
                    while let Some(c) = self.peek() {
                        if c == '\n' {
                            break;
                        }
                        self.advance();
                    }
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
                } else if self.match_char('>') {
                    self.add_token(TokenKind::ThickArrow, None);
                } else {
                    self.add_token(TokenKind::Equal, None);
                }
            }
            '!' => {
                if self.match_char('=') {
                    self.add_token(TokenKind::BangEqual, None);
                } else {
                    self.add_token(TokenKind::Bang, None);
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
            c if c.is_whitespace() && c != '\n' => {
                self.skip_whitespace();
            }
            '\n' => {
                self.infer_semicolon();
                self.line += 1;
                self.column = 1;
            }
            '"' => {
                self.lex_string(StringType::Normal)?;
            }
            'b' => {
                if self.match_char('"') {
                    self.lex_string(StringType::Byte)?;
                } else {
                    self.lex_identifier();
                }
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
                    .add_note("Siguro ito ay hindi parte ng sintax ng Tol")
                    .add_help("Subukan mo itong tanggalin"));
                }
            }
        }

        Ok(())
    }

    fn infer_semicolon(&mut self) {
        if let Some(tok) = self.parent_module.tokens.last()
            && matches!(
                tok.kind(),
                TokenKind::Identifier
                    | TokenKind::RightParen
                    | TokenKind::RightBracket
                    | TokenKind::IntLit
                    | TokenKind::FloatLit
                    | TokenKind::StringLit
                    | TokenKind::ByteStringLit
            )
        {
            self.start_column += 1;
            self.add_token(TokenKind::SemiColon, Some(";"));
        }
    }

    fn lex_identifier(&mut self) {
        while let Some(ch) = self.peek() {
            if self.is_identifier_continue(ch) {
                self.advance();
            } else {
                break;
            }
        }

        let lexeme = self.parent_module.source_code[self.start..self.current].to_string();

        match self.keywords.get(&lexeme) {
            Some(keyword_kind) => self.add_token(keyword_kind.clone(), Some(&lexeme)),
            None => self.add_token(TokenKind::Identifier, Some(&lexeme)),
        }
    }

    fn lex_number(&mut self) {
        let mut is_float = false;

        // Lex integer part
        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() || ch == '_' {
                self.advance();
            } else {
                break;
            }
        }

        #[allow(clippy::collapsible_if)]
        // Check for fractional part
        if let Some('.') = self.peek() {
            if let Some(next_ch) = self.peek_next() {
                if next_ch.is_ascii_digit() {
                    // Only treat as float if a digit follows the '.'
                    is_float = true;
                    self.advance(); // consume '.'

                    while let Some(ch) = self.peek() {
                        if ch.is_ascii_digit() || ch == '_' {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                }
                // else: '.' is not part of the number, leave it for another token
            }
        }

        let with_underscores = &self.parent_module.source_code[self.start..self.current];
        let without_underscores: String = with_underscores.chars().filter(|&c| c != '_').collect();

        let kind = if is_float {
            TokenKind::FloatLit
        } else {
            TokenKind::IntLit
        };

        self.add_token(kind, Some(&without_underscores));
    }

    fn lex_string(&mut self, string_type: StringType) -> Result<(), CompilerError> {
        let mut value = String::new();

        while let Some(ch) = self.peek() {
            match ch {
                '"' => {
                    self.advance(); // Consumes closing `"`
                    match string_type {
                        StringType::Byte => self.add_token(TokenKind::ByteStringLit, Some(&value)),
                        StringType::Normal => self.add_token(TokenKind::StringLit, Some(&value)),
                    };

                    return Ok(());
                }
                '\n' => {
                    self.line += 1;
                    self.column = 1;
                    value.push('\n');
                    self.advance();
                }
                '\\' => {
                    self.advance();
                    if let Some(esc) = self.advance() {
                        let unescaped = match esc {
                            'n' => '\n',
                            't' => '\t',
                            'r' => '\r',
                            '"' => '"',
                            '\\' => '\\',
                            other => other,
                        };
                        value.push(unescaped);
                    } else {
                        return Err(CompilerError::new(
                            "Ang sinulid ay hindi isinara",
                            ErrorKind::Error,
                            self.line,
                            self.column,
                        )
                        .add_help("Subukan mog maglagay ng `\"` sa huli"));
                    }
                }
                _ => {
                    value.push(ch);
                    self.advance();
                }
            }
        }

        Err(CompilerError::new(
            "Ang sinulid ay hindi isinara",
            ErrorKind::Error,
            self.line,
            self.start_column,
        ))
    }

    fn add_token(&mut self, kind: TokenKind, literal: Option<&str>) {
        let lexeme = match literal {
            Some(lxm) => lxm,
            None => &self.parent_module.source_code[self.start..self.current],
        };

        self.parent_module
            .tokens
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

    #[allow(dead_code)]
    fn is_ascii_identifier_start(&self, ch: char) -> bool {
        ch.is_ascii_alphabetic() || ch == '_'
    }

    #[allow(dead_code)]
    fn is_ascii_identifier_continue(&self, ch: char) -> bool {
        ch.is_ascii_alphanumeric() || ch == '_'
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
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

    fn peek(&self) -> Option<char> {
        self.parent_module.source_code[self.current..]
            .chars()
            .next()
    }

    // Peek at the character after the current one
    fn peek_next(&self) -> Option<char> {
        let mut chars_iter = self.parent_module.source_code[self.current..].chars();
        chars_iter.next()?;
        chars_iter.next()
    }

    // Consume the current character and advance
    fn advance(&mut self) -> Option<char> {
        if let Some(ch) = self.peek() {
            self.current += ch.len_utf8(); // advance the byte index
            self.column += 1;
            Some(ch)
        } else {
            None
        }
    }

    // Match a specific character
    fn match_char(&mut self, expected: char) -> bool {
        if let Some(ch) = self.peek()
            && ch == expected
        {
            self.advance();
            return true;
        }

        false
    }

    // Check if we are at the end of the input
    fn is_at_end(&self) -> bool {
        self.current >= self.parent_module.source_code.len()
    }

    pub fn has_error(&self) -> bool {
        self.has_error
    }
}
