use crate::{
    error::{CompilerError, ErrorKind},
    lexer::{token::Token, token_kind::TokenKind},
    parser::ast::{expr::Expr, stmt::Stmt},
    toltype::TolType,
};

pub mod ast;

pub struct Parser<'a> {
    tokens: &'a Vec<Token>,
    current: usize,
    source_path: &'a str,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a Vec<Token>, source_path: &'a str) -> Self {
        Parser {
            tokens,
            current: 0,
            source_path,
        }
    }

    pub fn parse(&mut self) -> Stmt {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            if self.peek().kind() == &TokenKind::Eof {
                break;
            }

            let statement = self.parse_statement();
            match statement {
                Ok(stmt) => statements.push(stmt),
                Err(e) => {
                    e.display(self.source_path);
                    self.synchronize();
                }
            }
        }

        Stmt::Program(statements)
    }

    fn parse_statement(&mut self) -> Result<Stmt, CompilerError> {
        match self.peek().kind() {
            TokenKind::Paraan => self.parse_par(),
            TokenKind::Ang => self.parse_ang(),
            TokenKind::Ibalik => self.parse_ibalik(),
            TokenKind::Bagay => self.parse_bagay(),
            _ => self.parse_expr_stmt(),
        }
    }

    fn parse_par(&mut self) -> Result<Stmt, CompilerError> {
        let par_tok = self
            .consume(
                TokenKind::Paraan,
                CompilerError::new(
                    &format!(
                        "Nag-asa ng `par`, pero nakita ay `{}`",
                        self.peek().lexeme()
                    ),
                    ErrorKind::Error,
                    self.peek().line(),
                    self.peek().column(),
                ),
            )?
            .clone();

        let par_identifier = self
            .consume(
                TokenKind::Identifier,
                CompilerError::new(
                    &format!(
                        "Nag-asa ng maiiba, pero nakita ay `{}`",
                        self.peek().lexeme()
                    ),
                    ErrorKind::Error,
                    self.peek().line(),
                    self.peek().column(),
                ),
            )?
            .clone();

        let params = self.parse_params()?;

        let mut return_type = TolType::Wala;
        if self.peek().kind() == &TokenKind::ThinArrow {
            self.advance(); // Consumes `->`
            return_type = self.parse_type()?;
        }

        let block = self.parse_block()?;

        Ok(Stmt::Par {
            par_identifier,
            params,
            return_type,
            block,
            line: par_tok.line(),
            column: par_tok.column(),
        })
    }

    fn parse_params(&mut self) -> Result<Vec<(Token, TolType)>, CompilerError> {
        let mut params = Vec::new();
        self.consume(
            TokenKind::LeftParen,
            CompilerError::new(
                &format!(
                    "Nag-asa ng `(` para simulan ang mga parameter, pero nakita ay `{}`",
                    self.peek().lexeme()
                ),
                ErrorKind::Error,
                self.peek().line(),
                self.peek().column(),
            )
            .with_help("Lagyan mo ng `(` dito para simulan ang pag deklara ng mga parameter"),
        )?;

        while self.peek().kind() != &TokenKind::RightParen {
            let param_identifier = self
                .consume(
                    TokenKind::Identifier,
                    CompilerError::new(
                        &format!(
                            "Nag-asa ng maiiba, pero nakita ay `{}`",
                            self.peek().lexeme()
                        ),
                        ErrorKind::Error,
                        self.peek().line(),
                        self.peek().column(),
                    ),
                )?
                .clone();

            self.consume(
                TokenKind::Colon,
                CompilerError::new(
                    &format!("Nag-asa ng `:`, pero nakita ay `{}`", self.peek().lexeme()),
                    ErrorKind::Error,
                    self.peek().line(),
                    self.peek().column(),
                )
                .with_help("Lagyan mo ng `:` dito")
                .with_note("Ang `:` ay ginagamit sa pag hiwalay ng tipo sa maiiba"),
            )?;

            let param_type = self.parse_type()?;

            if self.peek().kind() == &TokenKind::Comma {
                self.advance();
            } else if self.peek().kind() != &TokenKind::RightParen {
                return Err(CompilerError::new(
                    "Hindi naisarado ang `(` o hindi mo nilagyan ng `,`",
                    ErrorKind::Error,
                    self.peek().line(),
                    self.peek().column(),
                )
                .with_note("Ang `)` ay ginagamit sa pagsarado ng mga parameter, ang `,` naman ay ginagamit para ihiwalay ang mga parameter"));
            }

            params.push((param_identifier, param_type))
        }

        self.advance(); // Consumes `)`

        Ok(params)
    }

    fn parse_block(&mut self) -> Result<Expr, CompilerError> {
        let left_brace_tok = self
            .consume(
                TokenKind::LeftBrace,
                CompilerError::new(
                    &format!(
                        "Nag-asa ng `{{` para simulan ang isang bloke, pero nakita ay `{}`",
                        self.peek().lexeme()
                    ),
                    ErrorKind::Error,
                    self.peek().line(),
                    self.peek().column(),
                ),
            )?
            .clone();

        let mut statements = Vec::new();
        while !self.is_at_end() && self.peek().kind() != &TokenKind::RightBrace {
            statements.push(self.parse_statement()?);
        }

        if self.is_at_end() {
            return Err(CompilerError::new(
                "Hindi naisara ang `{{` dito",
                ErrorKind::Error,
                left_brace_tok.line(),
                left_brace_tok.column(),
            )
            .with_help("Isarado gamit ang `}}`"));
        }

        self.advance(); // Consumes `}`

        Ok(Expr::Block {
            statements,
            line: left_brace_tok.line(),
            column: left_brace_tok.column(),
        })
    }

    fn parse_ang(&mut self) -> Result<Stmt, CompilerError> {
        let ang_tok = self
            .consume(
                TokenKind::Ang,
                CompilerError::new(
                    &format!(
                        "Nag-asa ng `ang` para magdeklara ng bagong maiiba, pero nakita ay `{}`",
                        self.peek().lexeme()
                    ),
                    ErrorKind::Error,
                    self.peek().line(),
                    self.peek().column(),
                ),
            )?
            .clone();

        let mutable = matches!(self.peek().kind(), TokenKind::Maiba);

        let ang_identifier = self
            .consume(
                TokenKind::Identifier,
                CompilerError::new(
                    &format!(
                        "Nag-asa ng maiiba, pero nakita ay `{}`",
                        self.peek().lexeme()
                    ),
                    ErrorKind::Error,
                    self.peek().line(),
                    self.peek().column(),
                ),
            )?
            .clone();

        self.consume(
            TokenKind::Colon,
            CompilerError::new(
                &format!("Nag-asa ng `:`, pero nakita ay `{}`", self.peek().lexeme()),
                ErrorKind::Error,
                self.peek().line(),
                self.peek().column(),
            )
            .with_help("Lagyan mo ng `:` dito")
            .with_note("Ang `:` ay ginagamit sa pag hiwalay ng tipo sa maiiba"),
        )?;

        let ang_type = self.parse_type()?;

        self.consume(
            TokenKind::Equal,
            CompilerError::new(
                "Ang `=` ang maaari dito",
                ErrorKind::Error,
                self.peek().line(),
                self.peek().column(),
            ),
        )?;
        let rhs = self.parse_expression(0)?;

        self.consume(
            TokenKind::SemiColon,
            CompilerError::new(
                &format!("Nag-asa ng `;`, pero nakita ay `{}`", self.peek().lexeme()),
                ErrorKind::Error,
                self.peek().line(),
                self.peek().column(),
            )
            .with_help("Lagyan mo ng `;`"),
        )?;

        Ok(Stmt::Ang {
            mutable,
            ang_identifier,
            ang_type,
            rhs,
            line: ang_tok.line(),
            column: ang_tok.column(),
        })
    }

    fn parse_ibalik(&mut self) -> Result<Stmt, CompilerError> {
        let ibalik_tok = self
            .consume(
                TokenKind::Ibalik,
                CompilerError::new(
                    &format!("Nag-asa ng `ibalik`, nakita ay `{}`", self.peek().lexeme()),
                    ErrorKind::Error,
                    self.peek().line(),
                    self.peek().column(),
                ),
            )?
            .clone();

        let rhs = self.parse_expression(0)?;

        self.consume(
            TokenKind::SemiColon,
            CompilerError::new(
                &format!("Nag-asa ng `;`, nakita ay `{}`", self.peek().lexeme()),
                ErrorKind::Error,
                self.peek().line(),
                self.peek().column(),
            ),
        )?;

        Ok(Stmt::Ibalik {
            rhs,
            line: ibalik_tok.line(),
            column: ibalik_tok.column(),
        })
    }

    fn parse_bagay(&mut self) -> Result<Stmt, CompilerError> {
        self.consume(
            TokenKind::Bagay,
            CompilerError::new(
                &format!(
                    "Umaasa ng `bagay` pero nakita ay `{}`",
                    self.peek().lexeme()
                ),
                ErrorKind::Error,
                self.peek().line(),
                self.peek().column(),
            ),
        )?
        .clone();

        let bagay_identifier = self
            .consume(
                TokenKind::Identifier,
                CompilerError::new(
                    &format!(
                        "Umaasa ng pangalan para sa bagay na ito pero nakita ay `{}`",
                        self.peek().lexeme()
                    ),
                    ErrorKind::Error,
                    self.peek().line(),
                    self.peek().column(),
                ),
            )?
            .clone();

        let fields = self.parse_bagay_fields()?;

        Ok(Stmt::Bagay {
            bagay_identifier,
            fields,
        })
    }

    fn parse_bagay_fields(&mut self) -> Result<Vec<(Token, TolType)>, CompilerError> {
        self.consume(
            TokenKind::LeftBrace,
            CompilerError::new(
                &format!("Umaasa ng `{{` pero nakita ay `{}`", self.peek().lexeme()),
                ErrorKind::Error,
                self.peek().line(),
                self.peek().column(),
            ),
        )?;

        let mut fields = Vec::new();
        while self.peek().kind() != &TokenKind::RightBrace {
            let field_id = self
                .consume(
                    TokenKind::Identifier,
                    CompilerError::new(
                        &format!("Umaasa ng maiba pero nakita ay `{}`", self.peek().lexeme()),
                        ErrorKind::Error,
                        self.peek().line(),
                        self.peek().column(),
                    ),
                )?
                .clone();
            self.consume(
                TokenKind::Colon,
                CompilerError::new(
                    &format!("Umaasa ng `:` pero nakita ay `{}`", self.peek().lexeme()),
                    ErrorKind::Error,
                    self.peek().line(),
                    self.peek().column(),
                ),
            )?;
            let field_type = self.parse_type()?;

            if self.peek().kind() == &TokenKind::Comma {
                self.advance();
            } else if self.peek().kind() != &TokenKind::RightBrace {
                return Err(CompilerError::new(
                    &format!(
                        "Umaaasa ng `,` o `}}` pero nakita ay `{}`",
                        self.peek().lexeme()
                    ),
                    ErrorKind::Error,
                    self.peek().line(),
                    self.peek().column(),
                ));
            }

            fields.push((field_id, field_type));
        }

        self.advance(); // Consumes `}`

        Ok(fields)
    }

    fn parse_expr_stmt(&mut self) -> Result<Stmt, CompilerError> {
        let start_tok = self.peek().clone();
        let expr = self.parse_expression(0)?;

        self.consume(
            TokenKind::SemiColon,
            CompilerError::new(
                &format!("Nag-asa ng `;`, pero nakita ay `{}`", self.peek().lexeme()),
                ErrorKind::Error,
                self.peek().line(),
                self.peek().column(),
            )
            .with_help("Lagyan mo ng `;`"),
        )?;

        Ok(Stmt::ExprS {
            expr,
            line: start_tok.line(),
            column: start_tok.column(),
        })
    }

    fn parse_type(&mut self) -> Result<TolType, CompilerError> {
        // NOTE: Only works for primitives for now
        match self.peek().lexeme() {
            "i8" => {
                self.advance();
                Ok(TolType::I8)
            }
            "i16" => {
                self.advance();
                Ok(TolType::I16)
            }
            "i32" => {
                self.advance();
                Ok(TolType::I32)
            }
            "i64" => {
                self.advance();
                Ok(TolType::I64)
            }
            "u8" => {
                self.advance();
                Ok(TolType::U8)
            }
            "u16" => {
                self.advance();
                Ok(TolType::U16)
            }
            "u32" => {
                self.advance();
                Ok(TolType::U32)
            }
            "u64" => {
                self.advance();
                Ok(TolType::U64)
            }
            "lutang" => {
                self.advance();
                Ok(TolType::Lutang)
            }
            "dobletang" => {
                self.advance();
                Ok(TolType::DobleTang)
            }
            "bool," => {
                self.advance();
                Ok(TolType::Bool)
            }
            "kar" => {
                self.advance();
                Ok(TolType::Kar)
            }
            "wala" => {
                self.advance();
                Ok(TolType::Wala)
            }
            _ => {
                self.advance();
                // NOTE: Error to tagalog?
                Err(CompilerError::new(
                    &format!(
                        "`{}` ay hindi valid na tipo at hindi valid na simula ng isang tipo",
                        self.peek().lexeme()
                    ),
                    ErrorKind::Error,
                    self.peek().line(),
                    self.peek().column(),
                ))
            }
        }
    }

    fn parse_expression(&mut self, precedence: i32) -> Result<Expr, CompilerError> {
        let mut left = self.nud()?;

        while !self.is_at_end() {
            let op = self.peek().clone();
            if self.get_precedence(&op) <= precedence {
                break;
            }

            self.advance();
            left = self.led(&op, left)?;
        }

        Ok(left)
    }

    fn nud(&mut self) -> Result<Expr, CompilerError> {
        let current_tok = self.advance().clone();

        match current_tok.kind() {
            TokenKind::IntLit => Ok(Expr::IntLit(current_tok)),
            TokenKind::FloatLit => Ok(Expr::FloatLit(current_tok)),
            TokenKind::StringLit => Ok(Expr::StringLit(current_tok)),
            TokenKind::Identifier => {
                if self.peek().kind() == &TokenKind::LeftParen {
                    return self.parse_fncall(&current_tok);
                }

                Ok(Expr::Identifier(current_tok))
            }
            TokenKind::LeftParen => {
                let expr = self.parse_expression(0)?;
                self.consume(
                    TokenKind::RightParen,
                    CompilerError::new(
                        &format!("Nag-asa ng `)`, pero nakita ay `{}`", self.peek().lexeme()),
                        ErrorKind::Error,
                        self.peek().line(),
                        self.peek().column(),
                    )
                    .with_help("Lagyan mo ng `)`"),
                )?;
                Ok(expr)
            }
            TokenKind::At => {
                let callee = self.advance().clone();

                let fncall = self.parse_fncall(&callee)?;
                Ok(Expr::MagicFnCall {
                    fncall: Box::new(fncall),
                })
            }
            _ => Err(CompilerError::new(
                &format!(
                    "Nag-asa ng expression, pero nakita ay `{}`",
                    self.peek().lexeme()
                ),
                ErrorKind::Error,
                self.peek().line(),
                self.peek().column(),
            )),
        }
    }

    fn led(&mut self, op: &Token, left: Expr) -> Result<Expr, CompilerError> {
        let precedence = self.get_precedence(op);
        let right = self.parse_expression(precedence)?;

        Ok(Expr::Binary {
            op: op.clone(),
            left: Box::new(left),
            right: Box::new(right),
        })
    }

    fn parse_fncall(&mut self, callee: &Token) -> Result<Expr, CompilerError> {
        self.advance(); // Consumes `(`

        let mut args = Vec::new();
        while self.peek().kind() != &TokenKind::RightParen {
            args.push(self.parse_expression(0)?);

            if self.peek().kind() == &TokenKind::Comma {
                self.advance();
            } else if self.peek().kind() != &TokenKind::RightParen {
                return Err(CompilerError::new(
                    &format!("Nag-asa ng `,` o `)`, nakita ay `{}`", self.peek().lexeme()),
                    ErrorKind::Error,
                    self.peek().line(),
                    self.peek().column(),
                ));
            }
        }

        self.advance(); // Consumes the `)`

        Ok(Expr::FnCall {
            callee: callee.to_owned(),
            args,
        })
    }

    fn synchronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            if self.peek().kind() == &TokenKind::SemiColon {
                self.advance();
                return;
            }

            match self.peek().kind() {
                TokenKind::Paraan | TokenKind::Ang => return,
                _ => {}
            }

            self.advance();
        }
    }

    fn get_precedence(&self, op: &Token) -> i32 {
        match op.kind() {
            TokenKind::Plus | TokenKind::Minus => 1,
            TokenKind::Star | TokenKind::Slash => 2,
            _ => 0,
        }
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
            &self.tokens[self.current - 1]
        } else {
            panic!("Unexpected end of input")
        }
    }

    fn consume(
        &mut self,
        expected_kind: TokenKind,
        err: CompilerError,
    ) -> Result<&Token, CompilerError> {
        if !self.is_at_end() {
            if &expected_kind == self.peek().kind() {
                Ok(self.advance())
            } else {
                Err(err)
            }
        } else {
            panic!("Unexpected end of input");
        }
    }

    fn peek(&self) -> &Token {
        if !self.is_at_end() {
            &self.tokens[self.current]
        } else {
            panic!("Unexpected end of input")
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len()
    }
}

#[cfg(test)]
mod test {
    use crate::lexer::Lexer;

    use super::*;

    fn lex(source: &str) -> Vec<Token> {
        let mut lexer = Lexer::new(source, "test");
        lexer.lex().clone()
    }

    fn parse_expression(tokens: &Vec<Token>) -> Result<Expr, CompilerError> {
        let mut parser = Parser::new(tokens, "test");
        parser.parse_expression(0)
    }

    fn parse_program(tokens: &Vec<Token>) -> Stmt {
        let mut parser = Parser::new(tokens, "test");
        parser.parse()
    }

    #[test]
    fn test_expression() {
        let tokens = lex("20 + 10 / 30 - ((12 - 10) * 20)");
        let ast = parse_expression(&tokens);

        assert!(ast.is_ok());
        assert!(matches!(ast.as_ref().unwrap(), Expr::Binary { .. }));
        assert_eq!(
            format!("{}", ast.as_ref().unwrap()),
            "(- (+ 20 (/ 10 30)) (* (- 12 10) 20))"
        );
    }

    #[test]
    fn test_invalid_expression() {
        let tokens = lex("20 ++ 10 *! 50");
        let ast = parse_expression(&tokens);

        assert!(ast.is_err())
    }

    #[test]
    fn test_root() {
        let tokens = lex("");
        let ast = parse_program(&tokens);

        assert!(matches!(ast, Stmt::Program(_)));
    }

    #[test]
    fn test_program() {
        let program = "par una() {\
            ang x: i32 = 12;
            ang y: dobletang = 42; 
        }

        par idagdag(a: i32, b: i32) -> i32 {
            ang resulta: i32 = a + b;
        }";

        let tokens = lex(program);
        let ast = parse_program(&tokens);

        assert!(matches!(ast, Stmt::Program(_)));

        let statements = if let Stmt::Program(statements) = &ast {
            statements
        } else {
            &Vec::new()
        };

        assert!(matches!(statements[0], Stmt::Par { .. }));
        assert!(matches!(statements[1], Stmt::Par { .. }));

        let first_function = &statements[0];
        let second_function = &statements[1];

        if let Stmt::Par {
            par_identifier,
            params,
            return_type,
            block,
            ..
        } = first_function
        {
            assert_eq!(par_identifier.lexeme(), "una");
            assert_eq!(params.len(), 0);
            assert_eq!(return_type, &TolType::Wala);

            assert!(matches!(block, Expr::Block { .. }));
            if let Expr::Block { statements, .. } = block {
                assert!(matches!(statements[0], Stmt::Ang { .. }));
                assert!(matches!(statements[1], Stmt::Ang { .. }));
            }
        }

        if let Stmt::Par {
            par_identifier,
            params,
            return_type,
            block,
            ..
        } = second_function
        {
            assert_eq!(par_identifier.lexeme(), "idagdag");
            assert_eq!(params.len(), 2);
            assert_eq!(return_type, &TolType::I32);

            assert!(matches!(block, Expr::Block { .. }));
            if let Expr::Block { statements, .. } = block {
                assert!(matches!(statements[0], Stmt::Ang { .. }));
            }
        }
    }
}
