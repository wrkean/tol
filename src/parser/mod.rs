use crate::{
    error::{CompilerError, ErrorKind},
    lexer::{token::Token, token_kind::TokenKind},
    parser::{
        ast::{
            expr::Expr,
            stmt::{KungBranch, Stmt},
        },
        module::Module,
    },
    toltype::TolType,
};

pub mod ast;
pub mod module;

pub struct Parser<'a> {
    parent_module: &'a mut Module,
    current: usize,
    ast_id: usize,
    has_error: bool,
}

impl<'a> Parser<'a> {
    pub fn new(parent_module: &'a mut Module) -> Self {
        Parser {
            parent_module,
            current: 0,
            ast_id: 0,
            has_error: false,
        }
    }

    pub fn parse(&mut self) {
        let mut statements = std::mem::take(&mut self.parent_module.ast);
        while !self.is_at_end() {
            if self.peek().kind() == &TokenKind::Eof {
                break;
            }

            let statement = self.parse_statement();
            match statement {
                Ok(stmt) => statements.push(stmt),
                Err(e) => {
                    e.display(&self.parent_module.source_path);
                    self.has_error = true;
                    self.synchronize();
                }
            }
        }

        self.parent_module.ast = statements;
    }

    fn parse_statement(&mut self) -> Result<Stmt, CompilerError> {
        match self.peek().kind() {
            TokenKind::Paraan => self.parse_par(),
            TokenKind::Ang => {
                let stmt = self.parse_ang()?;
                self.consume(TokenKind::SemiColon, self.expect_err("`;`"))?;
                Ok(stmt)
            }
            TokenKind::Ibalik => {
                let stmt = self.parse_ibalik()?;
                self.consume(TokenKind::SemiColon, self.expect_err("`;`"))?;

                Ok(stmt)
            }
            TokenKind::Bagay => self.parse_bagay(),
            TokenKind::Itupad => self.parse_itupad(),
            TokenKind::Kung => self.parse_kung(),
            TokenKind::Sa => self.parse_sa(), // Pharsa?
            _ => {
                let expr_stmt = self.parse_expr_stmt()?;
                self.consume(TokenKind::SemiColon, self.expect_err("`;`"))?;

                Ok(expr_stmt)
            }
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
            block: Box::new(block),
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

    fn parse_block(&mut self) -> Result<Stmt, CompilerError> {
        let left_brace_tok = self
            .consume(TokenKind::LeftBrace, self.expect_err("`{`"))?
            .clone();

        let mut statements = Vec::new();
        while !self.is_at_end() && self.peek().kind() != &TokenKind::RightBrace {
            match self.parse_statement() {
                Ok(stmt) => statements.push(stmt),
                Err(e) => {
                    e.display(&self.parent_module.source_path);
                    self.synchronize_until(&[TokenKind::RightBrace]);
                }
            };
        }

        if self.is_at_end() {
            return Err(CompilerError::new(
                "Hindi naisarado ang `{`",
                ErrorKind::Error,
                left_brace_tok.line(),
                left_brace_tok.column(),
            ));
        } else {
            self.consume(TokenKind::RightBrace, self.expect_err("`}`"))?;
        }

        let id = self.ast_id;
        self.ast_id += 1;
        Ok(Stmt::Block {
            statements,
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

        let mutable = match self.peek().kind() {
            TokenKind::Maiba => {
                self.advance();
                true
            }
            _ => false,
        };

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
        // println!("{:?}", self.peek());
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
        while !self.is_at_end() && self.peek().kind() != &TokenKind::RightBrace {
            match self.parse_method() {
                Ok(method) => methods.push(method),
                Err(e) => {
                    e.display(&self.parent_module.source_path);
                    self.synchronize_until(&[TokenKind::RightBrace]);
                }
            }
        }

        if self.is_at_end() {
            return Err(CompilerError::new(
                "Hindi naisarado ang `{`",
                ErrorKind::Error,
                lb_tok.line(),
                lb_tok.column(),
            ));
        } else {
            self.consume(TokenKind::RightBrace, self.expect_err("`}`"))?;
        }

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
            block: Box::new(block),
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
            block: Box::new(block),
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
            "*" => {
                self.advance();

                match self.peek().kind() {
                    TokenKind::Maiba => {
                        self.advance();
                        let right_type = self.parse_type()?;

                        Ok(TolType::MutablePointer(Box::new(right_type)))
                    }
                    _ => {
                        let right_type = self.parse_type()?;

                        Ok(TolType::Pointer(Box::new(right_type)))
                    }
                }
            }
            _ => Ok(TolType::UnknownIdentifier(
                self.advance().lexeme().to_string(),
            )),
        }
    }

    fn parse_expression(&mut self, precedence: i32) -> Result<Expr, CompilerError> {
        // println!("{:#?}", self.peek());
        let mut left = self.nud()?;

        while !self.is_at_end() {
            let op = self.peek().clone();
            if self.get_op_info(&op).0 <= precedence {
                break;
            }

            self.advance();
            left = self.led(&op, left)?;
        }

        Ok(left)
    }

    fn nud(&mut self) -> Result<Expr, CompilerError> {
        let current_tok = self.peek().clone();

        // TODO: Add precedence for unary ops later
        match current_tok.kind() {
            TokenKind::IntLit => {
                self.advance();

                let id = self.ast_id;
                self.ast_id += 1;
                Ok(Expr::IntLit {
                    token: current_tok,
                    id,
                })
            }
            TokenKind::FloatLit => {
                self.advance();

                let id = self.ast_id;
                self.ast_id += 1;
                Ok(Expr::FloatLit {
                    token: current_tok,
                    id,
                })
            }
            TokenKind::StringLit => {
                self.advance();

                let id = self.ast_id;
                self.ast_id += 1;
                Ok(Expr::StringLit {
                    token: current_tok,
                    id,
                })
            }
            TokenKind::ByteStringLit => {
                self.advance();

                let id = self.ast_id;
                self.ast_id += 1;
                Ok(Expr::ByteStringLit {
                    token: current_tok,
                    id,
                })
            }
            TokenKind::Identifier => {
                self.advance();

                let id = self.ast_id;
                self.ast_id += 1;
                Ok(Expr::Identifier {
                    token: current_tok,
                    id,
                })
            }
            TokenKind::LeftParen => {
                self.advance();

                let expr = self.parse_expression(0)?;
                self.consume(
                    TokenKind::RightParen,
                    self.expect_err("`)`").add_help("Lagyan mo ng `)`"),
                )?;

                Ok(expr)
            }
            TokenKind::At => {
                self.advance();

                let name = self.advance().clone();

                self.consume(TokenKind::LeftParen, self.expect_err("`(`"))?;
                let args = self.parse_args(name.line(), name.column())?;

                let id = self.ast_id;
                self.ast_id += 1;
                Ok(Expr::MagicFnCall { name, args, id })
            }
            TokenKind::Amper => {
                self.advance();
                match self.peek().kind() {
                    TokenKind::Maiba => {
                        self.advance();
                        let right = self.parse_expression(0)?;

                        let id = self.ast_id;
                        self.ast_id += 1;
                        Ok(Expr::MutableAddressOf {
                            of: Box::new(right),
                            line: current_tok.line(),
                            column: current_tok.column(),
                            id,
                        })
                    }
                    _ => {
                        let right = self.parse_expression(0)?;

                        let id = self.ast_id;
                        self.ast_id += 1;
                        Ok(Expr::AddressOf {
                            of: Box::new(right),
                            line: current_tok.line(),
                            column: current_tok.column(),
                            id,
                        })
                    }
                }
            }
            TokenKind::Star => {
                self.advance();
                let right = self.parse_expression(0)?;

                let id = self.ast_id;
                self.ast_id += 1;
                Ok(Expr::Deref {
                    right: Box::new(right),
                    line: current_tok.line(),
                    column: current_tok.column(),
                    id,
                })
            }
            TokenKind::LeftBracket => {
                self.advance();
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

    fn led(&mut self, op: &Token, left: Expr) -> Result<Expr, CompilerError> {
        let (precedence, associativity) = self.get_op_info(op);

        let precedence = match associativity {
            Associativity::Left => precedence,
            Associativity::Right => precedence + 1,
            Associativity::None => {
                unreachable!("Invalid operators are checked by the main expression loop")
            }
        };

        match op.kind() {
            TokenKind::Dot => self.parse_member_access(left),
            TokenKind::LeftParen => self.parse_fncall(left, op.line(), op.column()),
            TokenKind::Bang => self.parse_struct_expr(left, op.line(), op.column()),
            TokenKind::ColonColon => self.parse_scope_resolution(left, op.line(), op.column()),
            TokenKind::DotDot => {
                let right = self.parse_expression(precedence)?;
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
                let right = self.parse_expression(precedence)?;
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
            TokenKind::Equal => {
                let right = self.parse_expression(precedence)?;
                let id = self.ast_id;
                self.ast_id += 1;
                Ok(Expr::Assign {
                    left: Box::new(left),
                    right: Box::new(right),
                    line: op.line(),
                    column: op.column(),
                    id,
                })
            }
            _ => {
                let right = self.parse_expression(precedence)?;
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

    fn parse_member_access(&mut self, left: Expr) -> Result<Expr, CompilerError> {
        let id = self.ast_id;
        self.ast_id += 1;

        let member = self.consume(TokenKind::Identifier, self.expect_err("pangalan"))?;
        Ok(Expr::MemberAccess {
            left: Box::new(left),
            member: member.clone(),
            line: member.line(),
            column: member.column(),
            id,
        })
    }

    fn parse_scope_resolution(
        &mut self,
        left: Expr,
        line: usize,
        column: usize,
    ) -> Result<Expr, CompilerError> {
        let field = self
            .consume(TokenKind::Identifier, self.expect_err("pangalan"))?
            .clone();

        let id = self.ast_id;
        self.ast_id += 1;
        Ok(Expr::ScopeResolution {
            left: Box::new(left),
            field,
            line,
            column,
            id,
        })
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

    fn parse_struct_expr(
        &mut self,
        callee: Expr,
        line: usize,
        column: usize,
    ) -> Result<Expr, CompilerError> {
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

        if self.is_at_end() {
            return Err(CompilerError::new(
                "Ang `(` ay di naisarado",
                ErrorKind::Error,
                line,
                column,
            ));
        } else {
            self.consume(TokenKind::RightParen, self.expect_err("`)`"))?;
        }

        let id = self.ast_id;
        self.ast_id += 1;
        Ok(Expr::Struct {
            callee: Box::new(callee),
            fields,
            line,
            column,
            id,
        })
    }

    fn parse_fncall(
        &mut self,
        callee: Expr,
        line: usize,
        column: usize,
    ) -> Result<Expr, CompilerError> {
        let args = self.parse_args(line, column)?;

        let id = self.ast_id;
        self.ast_id += 1;
        Ok(Expr::FnCall {
            callee: Box::new(callee),
            args,
            line,
            column,
            id,
        })
    }

    fn parse_args(&mut self, line: usize, column: usize) -> Result<Vec<Expr>, CompilerError> {
        let mut args = Vec::new();
        while !self.is_at_end() && self.peek().kind() != &TokenKind::RightParen {
            args.push(self.parse_expression(0)?);

            if self.peek().kind() == &TokenKind::Comma {
                self.advance();
            } else if self.peek().kind() != &TokenKind::RightParen {
                return Err(self.expect_err("`,` o `)`"));
            }
        }

        if self.is_at_end() {
            return Err(CompilerError::new(
                "Ang `(` ay di naisarado",
                ErrorKind::Error,
                line,
                column,
            ));
        } else {
            self.consume(TokenKind::RightParen, self.expect_err("`)`"))?;
        }

        Ok(args)
    }

    fn synchronize(&mut self) {
        if self.is_at_end() {
            return;
        }

        self.advance();

        while !self.is_at_end() {
            let previous = &self.parent_module.tokens[self.current - 1];
            if matches!(
                previous.kind(),
                TokenKind::SemiColon | TokenKind::RightBrace
            ) {
                return;
            }
            match self.peek().kind() {
                TokenKind::Paraan
                | TokenKind::Ang
                | TokenKind::Ibalik
                | TokenKind::Bagay
                | TokenKind::Kung
                | TokenKind::At
                | TokenKind::Itupad
                | TokenKind::Sa => return,
                _ => {}
            }

            self.advance();
        }
    }

    fn synchronize_until(&mut self, end_tokens: &[TokenKind]) {
        while !self.is_at_end() {
            if end_tokens.contains(self.peek().kind()) {
                return;
            }

            match self.peek().kind() {
                TokenKind::Paraan
                | TokenKind::Ang
                | TokenKind::Ibalik
                | TokenKind::Bagay
                | TokenKind::Kung
                | TokenKind::At
                | TokenKind::Itupad
                | TokenKind::Sa => return,
                _ => {
                    self.advance();
                }
            }
        }
    }

    fn get_op_info(&self, op: &Token) -> (i32, Associativity) {
        use Associativity::*;

        match op.kind() {
            TokenKind::Equal => (1, Right),
            TokenKind::DotDot | TokenKind::DotDotEqual => (2, Left),
            TokenKind::Plus | TokenKind::Minus => (3, Left),
            TokenKind::Star | TokenKind::Slash => (4, Left),
            TokenKind::Dot | TokenKind::ColonColon => (5, Left),
            TokenKind::LeftParen | TokenKind::Bang => (6, Left),
            _ => (0, Associativity::None),
        }
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
            &self.parent_module.tokens[self.current - 1]
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
            &self.parent_module.tokens[self.current]
        } else {
            panic!("Unexpected end of input");
        }
    }

    // fn peek_previous(&self) -> &Token {
    //     if self.current <= self.tokens.len() {
    //         &self.tokens[self.current - 1]
    //     } else {
    //         panic!("Unexpected end of input");
    //     }
    // }

    fn is_at_end(&self) -> bool {
        self.current >= self.parent_module.tokens.len() - 1
    }

    pub fn has_error(&self) -> bool {
        self.has_error
    }
}

enum Associativity {
    Left,
    Right,
    None, // Only for non-operators
}
