use crate::{
    error::{CompilerError, ErrorKind},
    lexer::{token::Token, token_kind::TokenKind},
    parser::ast::{
        expr::Expr,
        stmt::{KungBranch, Stmt},
    },
    toltype::TolType,
};

pub mod ast;

pub struct Parser<'a> {
    tokens: &'a Vec<Token>,
    current: usize,
    source_path: &'a str,
    ast_id: usize,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a Vec<Token>, source_path: &'a str) -> Self {
        Parser {
            tokens,
            current: 0,
            source_path,
            ast_id: 0,
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
            TokenKind::Ang => {
                let stmt = self.parse_ang();
                self.consume(TokenKind::SemiColon, self.expect_err("`;`"))?;

                stmt
            }
            TokenKind::Ibalik => {
                let stmt = self.parse_ibalik();
                self.consume(TokenKind::SemiColon, self.expect_err("`;`"))?;

                stmt
            }
            TokenKind::Bagay => self.parse_bagay(),
            TokenKind::Itupad => self.parse_itupad(),
            TokenKind::Kung => self.parse_kung(),
            TokenKind::Sa => self.parse_sa(), // Pharsa?
            _ => self.parse_expr_stmt(),
        }
    }

    fn parse_par(&mut self) -> Result<Stmt, CompilerError> {
        let par_tok = self
            .consume(TokenKind::Paraan, self.expect_err("par"))?
            .clone();

        let par_identifier = self
            .consume(TokenKind::Identifier, self.expect_err("pangalan"))?
            .clone();

        self.consume(
            TokenKind::LeftParen,
            self.expect_err("`(`")
                .add_help("Lagyan mo ng `(` dito para simulan ang pag deklara ng mga parameter"),
        )?;
        let params = self.parse_params()?;
        self.consume(
            TokenKind::RightParen,
            self.expect_err("`)`")
                .add_help("Lagyan mo ng `)` para tapusin ang listahan ng parameter"),
        )?;

        let mut return_type = TolType::Wala;
        if self.peek().kind() != &TokenKind::LeftBrace {
            return_type = self.parse_type()?;
        }

        let block = self.parse_block()?;

        let id = self.ast_id;
        self.ast_id += 1;
        Ok(Stmt::Par {
            par_identifier,
            params,
            return_type,
            block,
            line: par_tok.line(),
            column: par_tok.column(),
            id,
        })
    }

    fn parse_params(&mut self) -> Result<Vec<(Token, TolType)>, CompilerError> {
        let mut params = Vec::new();

        if self.peek().lexeme() == "ako" {
            params.push((self.advance().clone(), TolType::AkoType));
        }

        if self.peek().kind() == &TokenKind::Comma {
            self.advance();
        }

        while self.peek().kind() != &TokenKind::RightParen {
            let param_identifier = self
                .consume(TokenKind::Identifier, self.expect_err("pangalan"))?
                .clone();

            self.consume(
                TokenKind::Colon,
                self.expect_err("`:`")
                    .add_help("Lagyan mo ng `:` dito")
                    .add_note("Ang `:` ay ginagamit sa pag hiwalay ng tipo sa maiiba"),
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
                .add_note("Ang `)` ay ginagamit sa pagsarado ng mga parameter, ang `,` naman ay ginagamit para ihiwalay ang mga parameter"));
            }

            params.push((param_identifier, param_type))
        }

        Ok(params)
    }

    fn parse_block(&mut self) -> Result<Expr, CompilerError> {
        let left_brace_tok = self
            .consume(TokenKind::LeftBrace, self.expect_err("`{`"))?
            .clone();

        let mut statements = Vec::new();
        let mut block_value = None;
        while !self.is_at_end() && self.peek().kind() != &TokenKind::RightBrace {
            let statement = self.parse_statement()?;

            if let Stmt::ExprS { expr, .. } = &statement {
                if self.peek().kind() == &TokenKind::RightBrace {
                    block_value = Some(Box::new(expr.clone()));
                } else {
                    self.consume(TokenKind::SemiColon, self.expect_err("`;`"))?;
                    statements.push(statement);
                }
            } else {
                statements.push(statement);
            }
        }

        if self.is_at_end() {
            return Err(CompilerError::new(
                "Hindi naisara ang `{{` dito",
                ErrorKind::Error,
                left_brace_tok.line(),
                left_brace_tok.column(),
            )
            .add_help("Isarado gamit ang `}}`"));
        }

        self.advance(); // Consumes `}`

        let id = self.ast_id;
        self.ast_id += 1;
        Ok(Expr::Block {
            statements,
            block_value,
            line: left_brace_tok.line(),
            column: left_brace_tok.column(),
            id,
        })
    }

    fn parse_ang(&mut self) -> Result<Stmt, CompilerError> {
        let ang_tok = self
            .consume(
                TokenKind::Ang,
                self.expect_err("`ang`")
                    .add_note("Ginamit ang `ang` para mag-deklara ng bagong pangalan"),
            )?
            .clone();

        let mutable = matches!(self.peek().kind(), TokenKind::Maiba);

        let ang_identifier = self
            .consume(
                TokenKind::Identifier,
                self.expect_err("pangalan")
                    .add_note("Siguraduhing hindi keyword ang iyong nailagay"),
            )?
            .clone();

        let mut ang_type = TolType::Unknown;
        if self.peek().kind() == &TokenKind::Colon {
            self.advance();
            ang_type = self.parse_type()?;
        }

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

        let id = self.ast_id;
        self.ast_id += 1;
        Ok(Stmt::Ang {
            mutable,
            ang_identifier,
            ang_type,
            rhs,
            line: ang_tok.line(),
            column: ang_tok.column(),
            id,
        })
    }

    fn parse_ibalik(&mut self) -> Result<Stmt, CompilerError> {
        let ibalik_tok = self
            .consume(TokenKind::Ibalik, self.expect_err("`ibalik`"))?
            .clone();

        let rhs = self.parse_expression(0)?;

        let id = self.ast_id;
        self.ast_id += 1;
        Ok(Stmt::Ibalik {
            rhs,
            line: ibalik_tok.line(),
            column: ibalik_tok.column(),
            id,
        })
    }

    fn parse_bagay(&mut self) -> Result<Stmt, CompilerError> {
        self.consume(TokenKind::Bagay, self.expect_err("`bagay`"))?;

        let bagay_identifier = self
            .consume(TokenKind::Identifier, self.expect_err("pangalan"))?
            .clone();

        let fields = self.parse_bagay_fields()?;

        let id = self.ast_id;
        self.ast_id += 1;
        Ok(Stmt::Bagay {
            bagay_identifier,
            fields,
            id,
        })
    }

    fn parse_bagay_fields(&mut self) -> Result<Vec<(Token, TolType)>, CompilerError> {
        self.consume(TokenKind::LeftBrace, self.expect_err("`{`"))?;

        let mut fields = Vec::new();
        while self.peek().kind() != &TokenKind::RightBrace {
            let field_id = self
                .consume(TokenKind::Identifier, self.expect_err("pangalan"))?
                .clone();
            self.consume(TokenKind::Colon, self.expect_err("`:`"))?;
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

    fn parse_itupad(&mut self) -> Result<Stmt, CompilerError> {
        let itupad_tok = self
            .consume(TokenKind::Itupad, self.expect_err("`itupad`"))?
            .clone();

        let itupad_for = self.parse_type()?;

        let itupad_block = self.parse_itupad_block()?;

        let id = self.ast_id;
        self.ast_id += 1;
        Ok(Stmt::Itupad {
            itupad_for,
            itupad_block: Box::new(itupad_block),
            line: itupad_tok.line(),
            column: itupad_tok.column(),
            id,
        })
    }

    fn parse_itupad_block(&mut self) -> Result<Stmt, CompilerError> {
        let lb_tok = self
            .consume(TokenKind::LeftBrace, self.expect_err("`{`"))?
            .clone();

        let mut methods = Vec::new();
        while self.peek().kind() != &TokenKind::RightBrace {
            methods.push(self.parse_method()?);
        }

        self.advance(); // Consumes `}`

        let id = self.ast_id;
        self.ast_id += 1;
        Ok(Stmt::ItupadBlock {
            methods,
            line: lb_tok.line(),
            column: lb_tok.column(),
            id,
        })
    }

    fn parse_method(&mut self) -> Result<Stmt, CompilerError> {
        let par_tok = self
            .consume(TokenKind::Paraan, self.expect_err("`paraan`"))?
            .clone();

        let met_identifier = self
            .consume(TokenKind::Identifier, self.expect_err("pangalan"))?
            .clone();

        self.consume(TokenKind::LeftParen, self.expect_err("`(`"))?;
        let is_static = self.peek().lexeme() != "ako";
        let params = self.parse_params()?;
        self.advance(); // Consumes `)`

        let mut return_type = TolType::Wala;
        if self.peek().kind() != &TokenKind::LeftBrace {
            return_type = self.parse_type()?;
        }

        let block = self.parse_block()?;

        let id = self.ast_id;
        self.ast_id += 1;
        Ok(Stmt::Method {
            is_static,
            met_identifier,
            params,
            return_type,
            block,
            line: par_tok.line(),
            column: par_tok.column(),
            id,
        })
    }

    fn parse_expr_stmt(&mut self) -> Result<Stmt, CompilerError> {
        let start_tok = self.peek().clone();
        let expr = self.parse_expression(0)?;

        let id = self.ast_id;
        self.ast_id += 1;
        Ok(Stmt::ExprS {
            expr,
            line: start_tok.line(),
            column: start_tok.column(),
            id,
        })
    }

    fn parse_sa(&mut self) -> Result<Stmt, CompilerError> {
        let sa_tok = self
            .consume(TokenKind::Sa, self.expect_err("`sa`"))?
            .clone();

        let iterator = self.parse_expression(0)?;

        self.consume(TokenKind::ThickArrow, self.expect_err("`=>`"))?;
        let bind = self
            .consume(TokenKind::Identifier, self.expect_err("pangalan"))?
            .clone();

        let block = self.parse_block()?;

        let id = self.ast_id;
        self.ast_id += 1;
        Ok(Stmt::Sa {
            iterator,
            bind,
            block,
            line: sa_tok.line(),
            column: sa_tok.column(),
            id,
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
            "isukat" => {
                self.advance();
                Ok(TolType::ISukat)
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
            "usukat" => {
                self.advance();
                Ok(TolType::USukat)
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
            "[" => {
                self.advance();

                let mut len = None;
                if self.peek().kind() != &TokenKind::RightBracket {
                    let int_lit = self.consume(
                        TokenKind::IntLit,
                        self.expect_err("literal na integer")
                            .add_note("Literal na integer lang ang pwede sa loob ng []"),
                    )?;

                    len = match int_lit.lexeme().parse::<usize>() {
                        Ok(val) => Some(val),
                        Err(_) => {
                            return Err(CompilerError::new(
                                &format!("Nabigong gawing `usukat` ang {}", int_lit.lexeme()),
                                ErrorKind::Error,
                                int_lit.line(),
                                int_lit.column(),
                            )
                            .add_note("Siguraduhing hindi ito negatibong numero"));
                        }
                    };
                }

                self.consume(TokenKind::RightBracket, self.expect_err("`]`"))?;
                let elem_type = self.parse_type()?;

                Ok(TolType::Array(Box::new(elem_type), len))
            }
            _ => Ok(TolType::UnknownIdentifier(
                self.advance().lexeme().to_string(),
            )),
        }
    }

    fn parse_expression(&mut self, precedence: i32) -> Result<Expr, CompilerError> {
        let mut left = self.nud()?;

        while !self.is_at_end() {
            let previous_tok = self.peek_previous().clone();
            let op = self.peek().clone();
            if self.get_precedence(&op) <= precedence {
                break;
            }

            self.advance();
            left = self.led(&op, left, &previous_tok)?;
        }

        Ok(left)
    }

    fn nud(&mut self) -> Result<Expr, CompilerError> {
        let current_tok = self.advance().clone();

        match current_tok.kind() {
            TokenKind::IntLit => {
                let id = self.ast_id;
                self.ast_id += 1;
                Ok(Expr::IntLit {
                    token: current_tok,
                    id,
                })
            }
            TokenKind::FloatLit => {
                let id = self.ast_id;
                self.ast_id += 1;
                Ok(Expr::FloatLit {
                    token: current_tok,
                    id,
                })
            }
            TokenKind::StringLit => {
                let id = self.ast_id;
                self.ast_id += 1;
                Ok(Expr::StringLit {
                    token: current_tok,
                    id,
                })
            }
            TokenKind::Identifier => {
                if self.peek().kind() == &TokenKind::LeftParen {
                    return self.parse_fncall(&current_tok);
                } else if self.peek().kind() == &TokenKind::Bang {
                    return self.parse_struct_expr(&current_tok);
                }

                let id = self.ast_id;
                self.ast_id += 1;
                Ok(Expr::Identifier {
                    token: current_tok,
                    id,
                })
            }
            TokenKind::LeftParen => {
                let expr = self.parse_expression(0)?;
                self.consume(
                    TokenKind::RightParen,
                    self.expect_err("`)`").add_help("Lagyan mo ng `)`"),
                )?;

                Ok(expr)
            }
            TokenKind::At => {
                let callee = self.advance().clone();

                let fncall = self.parse_fncall(&callee)?;

                let id = self.ast_id;
                self.ast_id += 1;
                Ok(Expr::MagicFnCall {
                    fncall: Box::new(fncall),
                    id,
                })
            }
            TokenKind::LeftBracket => {
                let mut elements = Vec::new();
                while !self.is_at_end() && self.peek().kind() != &TokenKind::RightBracket {
                    elements.push(self.parse_expression(0)?);

                    if self.peek().kind() == &TokenKind::Comma {
                        self.advance();
                    } else if self.peek().kind() != &TokenKind::RightBracket {
                        return Err(self.expect_err("`]` o `,`"));
                    }
                }

                self.consume(TokenKind::RightBracket, self.expect_err("`}`"))?;

                let id = self.ast_id;
                self.ast_id += 1;
                Ok(Expr::Array {
                    elements,
                    line: current_tok.line(),
                    column: current_tok.column(),
                    id,
                })
            }
            _ => Err(self.expect_err("expresyon")),
        }
    }

    fn led(
        &mut self,
        op: &Token,
        left: Expr,
        tok_before_op: &Token,
    ) -> Result<Expr, CompilerError> {
        let precedence = self.get_precedence(op);
        let right = self.parse_expression(precedence)?;

        match op.kind() {
            TokenKind::Dot => match right {
                Expr::Identifier { token, .. } => {
                    let id = self.ast_id;
                    self.ast_id += 1;
                    Ok(Expr::FieldAccess {
                        left: Box::new(left),
                        member: token,
                        line: op.line(),
                        column: op.column(),
                        id,
                    })
                }
                // TODO: MethodCall
                Expr::FnCall { callee, args, .. } => {
                    let id = self.ast_id;
                    self.ast_id += 1;
                    Ok(Expr::MethodCall {
                        left: Box::new(left),
                        callee,
                        args,
                        line: op.line(),
                        column: op.column(),
                        id,
                    })
                }
                _ => Err(CompilerError::new(
                    "Ang nasa kanan ng `.` ay dapat pangalan o paraan",
                    ErrorKind::Error,
                    op.line(),
                    op.column(),
                )),
            },
            TokenKind::ColonColon => {
                match right {
                    Expr::Identifier { token, .. } => {
                        let id = self.ast_id;
                        self.ast_id += 1;
                        Ok(Expr::StaticFieldAccess {
                            left: tok_before_op.clone(),
                            field: token,
                            line: op.line(),
                            column: op.column(),
                            id,
                        })
                    }
                    // TODO: StaticMethodCall
                    Expr::FnCall { callee, args, .. } => {
                        let id = self.ast_id;
                        self.ast_id += 1;
                        Ok(Expr::StaticMethodCall {
                            left: TolType::UnknownIdentifier(tok_before_op.lexeme().to_string()),
                            callee,
                            args,
                            line: op.line(),
                            column: op.column(),
                            id,
                        })
                    }
                    _ => Err(CompilerError::new(
                        "Ang nasa kanan ng `::` ay dapat pangalan o paraan",
                        ErrorKind::Error,
                        op.line(),
                        op.column(),
                    )),
                }
            }
            TokenKind::DotDot => {
                let start = Box::new(left);
                let end = Box::new(right);
                let id = self.ast_id;
                self.ast_id += 1;
                Ok(Expr::RangeExclusive {
                    start,
                    end,
                    line: op.line(),
                    column: op.column(),
                    id,
                })
            }
            TokenKind::DotDotEqual => {
                let start = Box::new(left);
                let end = Box::new(right);
                let id = self.ast_id;
                self.ast_id += 1;
                Ok(Expr::RangeInclusive {
                    start,
                    end,
                    line: op.line(),
                    column: op.column(),
                    id,
                })
            }
            _ => {
                let id = self.ast_id;
                self.ast_id += 1;
                Ok(Expr::Binary {
                    op: op.clone(),
                    left: Box::new(left),
                    right: Box::new(right),
                    id,
                })
            }
        }
    }

    fn parse_kung(&mut self) -> Result<Stmt, CompilerError> {
        let kung_tok = self
            .consume(TokenKind::Kung, self.expect_err("`kung`"))?
            .clone();
        let condition = self.parse_expression(0)?;
        let block = self.parse_block()?;

        let mut branches = vec![KungBranch {
            condition: Some(condition),
            block,
        }];
        while self.peek().kind() == &TokenKind::KungDi {
            self.advance();
            let condition = self.parse_expression(0)?;
            branches.push(KungBranch {
                condition: Some(condition),
                block: self.parse_block()?,
            });
        }

        if self.peek().kind() == &TokenKind::KungWala {
            self.advance();
            branches.push(KungBranch {
                condition: None,
                block: self.parse_block()?,
            })
        }

        let id = self.ast_id;
        self.ast_id += 1;
        Ok(Stmt::Kung {
            branches,
            line: kung_tok.line(),
            column: kung_tok.column(),
            id,
        })
    }

    fn parse_struct_expr(&mut self, struct_name: &Token) -> Result<Expr, CompilerError> {
        self.consume(TokenKind::Bang, self.expect_err("`!`"))?;
        self.consume(TokenKind::LeftParen, self.expect_err("`(`"))?;

        let mut fields = Vec::new();
        while self.peek().kind() != &TokenKind::RightParen {
            let field_name = self
                .consume(TokenKind::Identifier, self.expect_err("pangalan"))?
                .clone();

            self.consume(TokenKind::Colon, self.expect_err("`:`"))?;

            let field_expr = self.parse_expression(0)?;

            if self.peek().kind() == &TokenKind::Comma {
                self.advance();
            } else if self.peek().kind() != &TokenKind::RightParen {
                return Err(self.expect_err("`}` o `,`"));
            }

            fields.push((field_name, field_expr));
        }

        self.advance();

        let id = self.ast_id;
        self.ast_id += 1;
        Ok(Expr::Struct {
            name: TolType::UnknownIdentifier(struct_name.lexeme().to_string()),
            fields,
            line: struct_name.line(),
            column: struct_name.column(),
            id,
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
                return Err(self.expect_err("`,` o `)`"));
            }
        }

        self.advance(); // Consumes the `)`

        let id = self.ast_id;
        self.ast_id += 1;
        Ok(Expr::FnCall {
            callee: callee.to_owned(),
            args,
            id,
        })
    }

    fn synchronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            match self.peek().kind() {
                TokenKind::Paraan
                | TokenKind::Ang
                | TokenKind::Ibalik
                | TokenKind::Bagay
                | TokenKind::Kung
                | TokenKind::At
                | TokenKind::Itupad => return,
                _ => {}
            }

            self.advance();
        }
    }

    fn get_precedence(&self, op: &Token) -> i32 {
        match op.kind() {
            TokenKind::DotDot | TokenKind::DotDotEqual => 1,
            TokenKind::Plus | TokenKind::Minus => 2,
            TokenKind::Star | TokenKind::Slash => 3,
            TokenKind::Dot | TokenKind::ColonColon => 4,
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

    // Handles the "Umasa ng X pero nakita ay Y" kind of errors
    fn expect_err(&self, expected: &str) -> CompilerError {
        CompilerError::new(
            &format!(
                "Umasa ng {} pero nakita ay {}",
                expected,
                self.peek().lexeme()
            ),
            ErrorKind::Error,
            self.peek().line(),
            self.peek().column(),
        )
    }

    fn peek(&self) -> &Token {
        if !self.is_at_end() {
            &self.tokens[self.current]
        } else {
            panic!("Unexpected end of input");
        }
    }

    fn peek_previous(&self) -> &Token {
        if self.current <= self.tokens.len() {
            &self.tokens[self.current - 1]
        } else {
            panic!("Unexpected end of input");
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
