use std::collections::{HashMap, HashSet, hash_map::Entry};

use crate::{
    error::{CompilerError, ErrorKind},
    lexer::{token::Token, token_kind::TokenKind},
    parser::ast::{expr::Expr, stmt::Stmt},
    symbol::Symbol,
    toltype::{TolType, type_info::TypeInfo},
};

pub struct SemanticAnalyzer<'a> {
    ast: &'a Stmt,
    source_path: &'a str,
    symbol_table: Vec<HashMap<String, Symbol>>,
    type_table: HashMap<TolType, TypeInfo>,
    has_error: bool,
    current_func_return_type: TolType,
    magic_funcs: HashSet<&'static str>,
}

impl<'a> SemanticAnalyzer<'a> {
    pub fn new(ast: &'a Stmt, source_path: &'a str) -> Self {
        let mut new_analyzer = Self {
            ast,
            source_path,
            symbol_table: vec![HashMap::new()],
            type_table: HashMap::new(),
            has_error: false,
            current_func_return_type: TolType::Unknown,
            magic_funcs: HashSet::from(["print", "println", "exit"]),
        };

        // Declare magic functions first
        new_analyzer.declare_magic_funcs();

        new_analyzer
    }

    pub fn has_error(&self) -> bool {
        self.has_error
    }

    pub fn analyze(&mut self) {
        let statements = match self.ast {
            Stmt::Program(stmts) => stmts,
            _ => panic!("ast did not start with a program node"),
        };

        for statement in statements {
            self.analyze_stmt(statement);
        }
    }

    fn analyze_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Ang {
                ang_identifier,
                ang_type,
                rhs,
                line,
                column,
                ..
            } => {
                if let Err(e) = self.analyze_ang(ang_identifier, ang_type, rhs, line, column) {
                    self.has_error = true;
                    e.display(self.source_path);
                }
            }
            Stmt::Par {
                par_identifier,
                params,
                return_type,
                block,
                // line,
                // column,
                ..
            } => {
                if let Err(e) = self.analyze_par(par_identifier, params, return_type, block) {
                    self.has_error = true;
                    e.display(self.source_path);
                }
            }
            Stmt::Ibalik { rhs, line, column } => {
                if let Err(e) = self.analyze_ibalik(rhs, line, column) {
                    self.has_error = true;
                    e.display(self.source_path);
                }
            }
            Stmt::ExprS { expr, .. } => {
                if let Err(e) = self.analyze_expression(expr) {
                    self.has_error = true;
                    e.display(self.source_path);
                }
            }
            Stmt::Bagay {
                bagay_identifier,
                fields,
            } => {
                if let Err(e) = self.analyze_bagay(bagay_identifier, fields) {
                    self.has_error = true;
                    e.display(self.source_path);
                }
            }
            Stmt::Program(_) => {}
        };
    }

    fn analyze_ang(
        &mut self,
        ang_identifier: &Token,
        ang_type: &TolType,
        rhs: &Expr,
        line: &usize,
        column: &usize,
    ) -> Result<(), CompilerError> {
        let ang_type = if let TolType::UnknownIdentifier(id) = ang_type {
            self.resolve_type(id, *line, *column)?
        } else {
            ang_type.clone()
        };

        let rhs_type = self.analyze_expression(rhs)?;
        if !rhs_type.is_assignment_compatible(&ang_type) {
            return Err(CompilerError::new(
                &format!(
                    "Ang tipong `{}` ay hindi pwede ilagay sa `{}`",
                    rhs_type, ang_type,
                ),
                ErrorKind::Error,
                *line,
                *column,
            ));
        }

        let var_symbol = Symbol::VarSymbol {
            name: ang_identifier.lexeme().to_string(),
            tol_type: ang_type.clone(),
        };

        if !self.declare_symbol(ang_identifier.lexeme(), var_symbol) {
            let err = self.declared_in_scope_err(ang_identifier);

            if let Some(func) = self.magic_funcs.get(ang_identifier.lexeme()) {
                err.add_note(&format!("Ang `{func}` ay isang paraan na may 'mahika'."))
                    .add_note("Ang paraan ay may mahika kung kilala ito ng compiler")
            } else {
                err
            };
        }

        Ok(())
    }

    fn resolve_type(&self, id: &str, line: usize, column: usize) -> Result<TolType, CompilerError> {
        self.type_table
            .get(&TolType::UnknownIdentifier(id.to_string()))
            .map(|t| t.kind.clone())
            .ok_or(CompilerError::new(
                &format!("Hindi valid na tipo ang `{}`", id),
                ErrorKind::Error,
                line,
                column,
            ))
    }

    fn analyze_par(
        &mut self,
        par_identifier: &Token,
        params: &[(Token, TolType)],
        return_type: &TolType,
        block: &Expr,
        // line: &usize,
        // column: &usize,
    ) -> Result<(), CompilerError> {
        self.enter_scope();

        // Resolve types
        let resolved_params: Vec<_> = params
            .iter()
            .map(|(tok, ty)| {
                let resolved_ty = if let TolType::UnknownIdentifier(id) = ty {
                    self.resolve_type(id, tok.line(), tok.column())?
                } else {
                    ty.clone()
                };

                Ok((tok.clone(), resolved_ty))
            })
            .collect::<Result<_, CompilerError>>()?;

        let resolved_return = if let TolType::UnknownIdentifier(id) = return_type {
            self.resolve_type(id, par_identifier.line(), par_identifier.column())?
        } else {
            return_type.clone()
        };

        let param_types: Vec<TolType> = resolved_params.iter().map(|(_, ty)| ty.clone()).collect();
        let par_symbol = Symbol::ParSymbol {
            name: par_identifier.lexeme().to_string(),
            param_types,
            return_type: resolved_return.clone(),
        };

        if !self.declare_symbol(par_identifier.lexeme(), par_symbol) {
            return Err(CompilerError::new(
                &format!(
                    "`{}` ay na-ideklara na sa kasalukuyang sakop",
                    par_identifier.lexeme()
                ),
                ErrorKind::Error,
                par_identifier.line(),
                par_identifier.column(),
            ));
        }

        self.current_func_return_type = resolved_return.to_owned();
        self.analyze_expression(block)?;

        self.exit_scope();
        Ok(())
    }

    fn analyze_ibalik(
        &mut self,
        rhs: &Expr,
        line: &usize,
        column: &usize,
    ) -> Result<(), CompilerError> {
        let return_type = self.analyze_expression(rhs)?;

        if self.current_func_return_type == TolType::Unknown && return_type != TolType::Wala {
            return Err(CompilerError::new(
                "Hindi pwede magbalik sa labas ng paraan",
                ErrorKind::Error,
                *line,
                *column,
            ));
        }

        if !return_type.is_assignment_compatible(&self.current_func_return_type) {
            return Err(CompilerError::new(
                &format!(
                    "Hindi pwede mag return ng `{}` dahil ang kasalukuyang paraan ay umaasa ng `{}`",
                    return_type, self.current_func_return_type
                ),
                ErrorKind::Error,
                *line,
                *column,
            ));
        }

        Ok(())
    }

    pub fn analyze_bagay(
        &mut self,
        bagay_identifier: &Token,
        fields: &[(Token, TolType)],
    ) -> Result<(), CompilerError> {
        let bagay_name = bagay_identifier.lexeme();
        let bagay_type = TolType::Bagay(bagay_name.to_string());

        let bagay_symbol = Symbol::BagaySymbol {
            name: bagay_name.to_string(),
        };
        if !self.declare_symbol(bagay_name, bagay_symbol) {
            return Err(self.declared_in_scope_err(bagay_identifier));
        }

        // Forward declare the type in the type table, this
        // allows fields to have indirect recursive types
        match self.type_table.entry(bagay_type.clone()) {
            Entry::Occupied(_) => {
                return Err(CompilerError::new(
                    &format!(
                        "Hindi marehistro ang `{}` dahil may ganitong tipo na",
                        bagay_name
                    ),
                    ErrorKind::Error,
                    bagay_identifier.line(),
                    bagay_identifier.column(),
                ));
            }
            Entry::Vacant(entry) => {
                entry.insert(TypeInfo {
                    kind: bagay_type.clone(),
                    fields: HashMap::new(), // Empty for now
                });
            }
        }

        let resolved_fields: Vec<_> = fields
            .iter()
            .map(|(tok, ty)| {
                let resolved_ty = if let TolType::UnknownIdentifier(id) = ty {
                    self.resolve_type(id, tok.line(), tok.column())?
                } else {
                    ty.clone()
                };
                Ok((tok.clone(), resolved_ty))
            })
            .collect::<Result<_, CompilerError>>()?;

        self.enter_scope();
        for (tok, ty) in &resolved_fields {
            if !self.declare_symbol(
                tok.lexeme(),
                Symbol::VarSymbol {
                    name: tok.lexeme().to_string(),
                    tol_type: ty.clone(),
                },
            ) {
                self.exit_scope();
                return Err(self.declared_in_scope_err(tok));
            }
        }
        self.exit_scope();

        let mut field_map = HashMap::new();
        for (tok, ty) in &resolved_fields {
            let field_name = tok.lexeme().to_string();

            if field_map.contains_key(&field_name) {
                return Err(CompilerError::new(
                    &format!("Duplicate field `{}` in `{}`", field_name, bagay_name),
                    ErrorKind::Error,
                    tok.line(),
                    tok.column(),
                ));
            }

            if matches!(ty, TolType::Bagay(name) if name == bagay_name) {
                return Err(CompilerError::new(
                    &format!(
                        "Ang `{}` ay hindi maaaring maglaman ng sarili nito nang direkta",
                        bagay_name
                    ),
                    ErrorKind::Error,
                    tok.line(),
                    tok.column(),
                ));
            }

            field_map.insert(field_name, ty.clone());
        }

        // Fill in the previously forward-declared entry
        if let Some(type_info) = self.type_table.get_mut(&bagay_type) {
            type_info.fields = field_map;
        }

        Ok(())
    }

    fn analyze_expression(&mut self, expr: &Expr) -> Result<TolType, CompilerError> {
        match expr {
            Expr::IntLit(_) => Ok(TolType::UnsizedInt),
            Expr::FloatLit(_) => Ok(TolType::UnsizedFloat),
            Expr::StringLit(_) => Ok(TolType::Sinulid),
            Expr::Identifier(tok) => match self.lookup_symbol(tok.lexeme()) {
                Some(s) => Ok(s.get_type().to_owned()),
                None => Err(CompilerError::new(
                    &format!("`{}` ay hindi pa na-ideklara", tok.lexeme()),
                    ErrorKind::Error,
                    tok.line(),
                    tok.column(),
                )),
            },
            Expr::Binary { op, left, right } => match op.kind() {
                TokenKind::Plus | TokenKind::Minus | TokenKind::Star | TokenKind::Slash => {
                    let left_type = self.analyze_expression(left)?;
                    let right_type = self.analyze_expression(right)?;

                    if !left_type.is_arithmetic_compatible(&right_type) {
                        Err(CompilerError::new(
                            &format!(
                                "Hindi pwede gawin ang `{}` na operasyon sa `{}` at `{}`",
                                op.lexeme(),
                                left_type,
                                right_type
                            ),
                            ErrorKind::Error,
                            op.line(),
                            op.column(),
                        ))
                    } else {
                        Ok(left_type)
                    }
                }
                _ => Err(CompilerError::new(
                    &format!("Hindi tamang operator `{}`", op.lexeme()),
                    ErrorKind::Error,
                    op.line(),
                    op.column(),
                )),
            },
            Expr::Block {
                statements,
                // line,
                // column,
                ..
            } => {
                self.enter_scope();
                for statement in statements {
                    self.analyze_stmt(statement);
                }
                self.exit_scope();
                Ok(TolType::Unknown)
            }
            Expr::FnCall { callee, args } => {
                let symbol = match self.lookup_symbol(callee.lexeme()) {
                    Some(s) => s,
                    None => {
                        return Err(CompilerError::new(
                            &format!("Ang `{}` ay hindi na-ideklara", callee.lexeme()),
                            ErrorKind::Error,
                            callee.line(),
                            callee.column(),
                        ));
                    }
                };

                if let Symbol::ParSymbol {
                    param_types,
                    return_type,
                    ..
                } = symbol
                {
                    if args.len() < param_types.len() {
                        return Err(CompilerError::new(
                            &format!(
                                "`{}` lang ang argumento na nailagay, nag-asa ng `{}`",
                                args.len(),
                                param_types.len()
                            ),
                            ErrorKind::Error,
                            callee.line(),
                            callee.column(),
                        ));
                    }

                    if args.len() > param_types.len() {
                        return Err(CompilerError::new(
                            &format!(
                                "`{}` ang nailagay na argumento, nag-asa lang ng `{}`",
                                args.len(),
                                param_types.len()
                            ),
                            ErrorKind::Error,
                            callee.line(),
                            callee.column(),
                        ));
                    }

                    let return_type_ = return_type.clone();
                    let param_types_ = param_types.clone();

                    let mut arg_types = Vec::with_capacity(args.len());
                    for expr in args {
                        arg_types.push(self.analyze_expression(expr)?);
                    }

                    // if arg_types != param_types_ {
                    //     return Err(CompilerError::new(
                    //         "Magkaiba ang tipo ng argumento sa mga parametro",
                    //         ErrorKind::Error,
                    //         callee.line(),
                    //         callee.column(),
                    //     ));
                    // }
                    for (arg, param) in arg_types.iter().zip(&param_types_) {
                        if !arg.is_assignment_compatible(param) {
                            return Err(CompilerError::new(
                                &format!("Hindi pwede ilagay ang {arg} sa {param}"),
                                ErrorKind::Error,
                                callee.line(),
                                callee.column(),
                            ));
                        }
                    }

                    return Ok(return_type_);
                }
                panic!("The symbol is not declared as `par` symbol");
            }
            Expr::MagicFnCall { fncall } => self.analyze_expression(fncall),
            Expr::FieldAccess {
                left,
                member,
                line,
                column,
            } => {
                let left_type = self.analyze_expression(left)?;

                match self.type_table.get(&left_type) {
                    Some(type_info) => match type_info.fields.get(member.lexeme()) {
                        Some(toltype) => Ok(toltype.to_owned()),
                        None => Err(CompilerError::new(
                            &format!(
                                "Ang `{}` ay hindi kabilamg sa `{}` na tipo",
                                member.lexeme(),
                                &left_type
                            ),
                            ErrorKind::Error,
                            member.line(),
                            member.column(),
                        )),
                    },
                    None => Err(CompilerError::new(
                        &format!("Walang miyembro ang `{}`", &left_type),
                        ErrorKind::Error,
                        *line,
                        *column,
                    )),
                }
            } // Expr::MethodCall {
              //     left,
              //     method,
              //     args,
              //     line,
              //     column,
              // } => {}
        }
    }

    fn declare_magic_funcs(&mut self) {
        let magic_symbols = vec![
            (
                "print",
                Symbol::ParSymbol {
                    name: "print".to_string(),
                    param_types: vec![TolType::Sinulid],
                    return_type: TolType::Wala,
                },
            ),
            (
                "println",
                Symbol::ParSymbol {
                    name: "println".to_string(),
                    param_types: vec![TolType::Sinulid],
                    return_type: TolType::Wala,
                },
            ),
            (
                "alis",
                Symbol::ParSymbol {
                    name: "alis".to_string(),
                    param_types: vec![TolType::I32],
                    return_type: TolType::Wala,
                },
            ),
        ];

        for (name, sym) in magic_symbols {
            self.declare_symbol(name, sym);
        }
    }

    fn lookup_symbol(&self, name: &str) -> Option<&Symbol> {
        for scope in self.symbol_table.iter().rev() {
            if let Some(s) = scope.get(name) {
                return Some(s);
            }
        }

        None
    }

    /// Returns true if the key did not exist (means it is declared successfully), returns false otherwise.
    fn declare_symbol(&mut self, name: &str, symbol: Symbol) -> bool {
        let current_scope = self.symbol_table.last_mut().unwrap();

        if !current_scope.contains_key(name) {
            current_scope.insert(name.to_string(), symbol);
            true
        } else {
            false
        }
    }

    fn declared_in_scope_err(&self, name: &Token) -> CompilerError {
        CompilerError::new(
            &format!(
                "Ang {} ay naideklara na sa kasalukuyang sakop",
                name.lexeme()
            ),
            ErrorKind::Error,
            name.line(),
            name.column(),
        )
    }

    fn enter_scope(&mut self) {
        self.symbol_table.push(HashMap::new());
    }

    fn exit_scope(&mut self) {
        let _ = self.symbol_table.pop();
    }
}

#[cfg(test)]
mod test {
    use crate::{lexer::Lexer, parser::Parser};

    use super::*;

    fn lex(source: &str) -> Vec<Token> {
        let mut lexer = Lexer::new(source, "test");

        lexer.lex().clone()
    }

    fn parse(tokens: &Vec<Token>) -> Stmt {
        let mut parser = Parser::new(tokens, "test");

        parser.parse()
    }

    #[test]
    fn test_analyze_assignment() {
        let code = "ang x: i32 = 15; ang y: i32 = x;";

        let tokens = lex(code);
        let ast = parse(&tokens);
        let mut analyzer = SemanticAnalyzer::new(&ast, "None");
        analyzer.analyze();
        assert!(!analyzer.has_error());
    }

    #[test]
    fn test_previously_declared_variable() {
        let code = "ang x: i32 = 15; ang y: i32 = 15;";

        let tokens = lex(code);
        let ast = parse(&tokens);
        let mut analyzer = SemanticAnalyzer::new(&ast, "None");
        analyzer.analyze();
        assert!(!analyzer.has_error());
    }

    #[test]
    fn test_type_mismatch_assignment() {
        let code = "ang x: i32 = 15.5;";
        let code2 = "ang x: i32 = 12; ang y: i8 = x;";

        let tokens = lex(code);
        let ast = parse(&tokens);
        let mut analyzer = SemanticAnalyzer::new(&ast, "None");
        analyzer.analyze();

        let tokens2 = lex(code2);
        let ast2 = parse(&tokens2);
        let mut analyzer2 = SemanticAnalyzer::new(&ast2, "None");
        analyzer2.analyze();

        assert!(analyzer.has_error());
        assert!(analyzer2.has_error());
    }

    #[test]
    fn test_undeclared_variable() {
        let code = "ang x: i32 = y;";

        let tokens = lex(code);
        let ast = parse(&tokens);
        let mut analyzer = SemanticAnalyzer::new(&ast, "None");
        analyzer.analyze();
        assert!(analyzer.has_error());
    }

    #[test]
    fn test_variable_scoping() {
        let code = "par dummy() {\n    ang y: i32 = 10;\n}\nang x: i32 = y;";

        let tokens = lex(code);
        let ast = parse(&tokens);
        let mut analyzer = SemanticAnalyzer::new(&ast, "None");
        analyzer.analyze();
        assert!(analyzer.has_error());
    }

    #[test]
    fn test_type_mismatch_binary_operation() {
        let code = "ang x: i32 = 10;\nang y: dobletang = 12.5;\nang z: i32 = x + y;";

        let tokens = lex(code);
        let ast = parse(&tokens);
        let mut analyzer = SemanticAnalyzer::new(&ast, "None");
        analyzer.analyze();
        assert!(analyzer.has_error());
    }

    #[test]
    fn test_incompatible_return_type() {
        let code = "par test() -> i32 { ibalik 15.5; }";
        let code2 = "par test() -> dobletang { ibalik 15; }";
        let code3 = "par test() -> i32 { ibalik 15; }";

        let tokens = lex(code);
        let ast = parse(&tokens);
        let mut analyzer = SemanticAnalyzer::new(&ast, "None");
        analyzer.analyze();

        let tokens2 = lex(code2);
        let ast2 = parse(&tokens2);
        let mut analyzer2 = SemanticAnalyzer::new(&ast2, "None");
        analyzer2.analyze();

        let tokens3 = lex(code3);
        let ast3 = parse(&tokens3);
        let mut analyzer3 = SemanticAnalyzer::new(&ast3, "None");
        analyzer3.analyze();

        assert!(analyzer.has_error());
        assert!(analyzer2.has_error());
        assert!(!analyzer3.has_error());
    }
}
