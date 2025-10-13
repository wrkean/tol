use std::collections::{HashMap, hash_map::Entry};

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
    type_table: HashMap<String, TypeInfo>,
    has_error: bool,
    current_func_return_type: TolType,
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
        };

        // Declare magic functions first
        new_analyzer.declare_magic_funcs();

        new_analyzer
    }

    pub fn analyze(&mut self) {
        let statements = match self.ast {
            Stmt::Program(stmts) => stmts,
            _ => panic!("ast did not start with a program node"),
        };

        // Collect type declarations
        for stmt in statements {
            if let Stmt::Bagay {
                bagay_identifier,
                fields,
            } = stmt
            {
                self.analyze_bagay(bagay_identifier, fields)
                    .unwrap_or_else(|e| e.display(self.source_path));
            }
        }

        // Second pass: analyze everything else
        for stmt in statements.iter() {
            if !matches!(stmt, Stmt::Bagay { .. }) {
                self.analyze_stmt(stmt);
            }
        }

        // println!("{:?}", self.type_table.keys());
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
            } => self
                .analyze_ang(ang_identifier, ang_type, rhs, line, column)
                .unwrap_or_else(|e| {
                    self.has_error = true;
                    e.display(self.source_path);
                }),
            Stmt::Par {
                par_identifier,
                params,
                return_type,
                block,
                // line,
                // column,
                ..
            } => self
                .analyze_par(par_identifier, params, return_type, block)
                .unwrap_or_else(|e| {
                    self.has_error = true;
                    e.display(self.source_path);
                }),
            Stmt::Ibalik { rhs, line, column } => {
                self.analyze_ibalik(rhs, line, column).unwrap_or_else(|e| {
                    self.has_error = true;
                    e.display(self.source_path);
                })
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
            } => self
                .analyze_bagay(bagay_identifier, fields)
                .unwrap_or_else(|e| {
                    self.has_error = true;
                    e.display(self.source_path);
                }),
            Stmt::Itupad {
                itupad_for,
                itupad_block,
                line,
                column,
            } => {
                if let Err(e) = self.analyze_itupad(itupad_for, itupad_block, *line, *column) {
                    self.has_error = true;
                    e.display(self.source_path);
                }
            }
            // TODO: analyze ts
            Stmt::ItupadBlock { .. } => {}
            Stmt::Method { .. } => {}
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
        let ang_type = self.resolve_type(ang_type, *line, *column)?;

        let rhs_type = self.analyze_expression(rhs)?;
        // println!("{:?}, {:?}", rhs_type, ang_type);
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

        let var_symbol = Symbol::Var {
            name: ang_identifier.lexeme().to_string(),
            tol_type: ang_type.clone(),
        };

        if !self.declare_symbol(ang_identifier.lexeme(), var_symbol) {
            return Err(self.declared_in_scope_err(ang_identifier));
        }

        Ok(())
    }

    fn resolve_type(
        &self,
        type_to_resolve: &TolType,
        line: usize,
        column: usize,
    ) -> Result<TolType, CompilerError> {
        match type_to_resolve {
            TolType::UnknownIdentifier(id) => self
                .type_table
                .get(id)
                .map(|t| t.kind.clone())
                .ok_or(CompilerError::new(
                    &format!("Hindi valid na tipo ang `{}`", id),
                    ErrorKind::Error,
                    line,
                    column,
                )),
            _ => Ok(type_to_resolve.clone()),
        }
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
        let resolved_params: Vec<_> = params
            .iter()
            .map(|(tok, ty)| {
                let resolved_ty = self.resolve_type(ty, tok.line(), tok.column())?;

                Ok((tok.clone(), resolved_ty))
            })
            .collect::<Result<_, CompilerError>>()?;

        let resolved_return =
            self.resolve_type(return_type, par_identifier.line(), par_identifier.column())?;

        let param_types: Vec<TolType> = resolved_params.iter().map(|(_, ty)| ty.clone()).collect();

        let par_symbol = Symbol::Paraan {
            name: par_identifier.lexeme().to_string(),
            param_types,
            return_type: resolved_return.clone(),
        };

        if !self.declare_symbol(par_identifier.lexeme(), par_symbol) {
            return Err(self.declared_in_scope_err(par_identifier));
        }

        self.enter_scope();
        for (tok, ty) in &resolved_params {
            let param_symbol = Symbol::Var {
                name: tok.lexeme().to_string(),
                tol_type: ty.clone(),
            };

            if !self.declare_symbol(tok.lexeme(), param_symbol) {
                return Err(self.declared_in_scope_err(tok));
            }
        }
        self.current_func_return_type = resolved_return.clone();
        let last_expr_type = self.analyze_expression(block)?;
        if !last_expr_type.is_assignment_compatible(&self.current_func_return_type) {
            return Err(CompilerError::new(
                &format!(
                    "Hindi pwede mag return ng `{}` dahil ang kasalukuyang paraan ay umaasa ng `{}`",
                    return_type, self.current_func_return_type
                ),
                ErrorKind::Error,
                par_identifier.line(),
                par_identifier.column(),
            ));
        }
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

        if self.current_func_return_type == TolType::Unknown {
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

        let bagay_symbol = Symbol::Bagay {
            name: bagay_name.to_string(),
        };
        if !self.declare_symbol(bagay_name, bagay_symbol) {
            return Err(self.declared_in_scope_err(bagay_identifier));
        }

        // Forward declare the type in the type table, this
        // allows fields to have indirect recursive types
        let bagay_type = TolType::Bagay(bagay_name.to_string());
        match self.type_table.entry(bagay_name.to_string()) {
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
                    fields: HashMap::new(),
                    methods: HashMap::new(),
                });
            }
        }

        let resolved_fields: Vec<(Token, TolType)> = fields
            .iter()
            .map(|(tok, ty)| {
                let resolved_ty = self.resolve_type(ty, tok.line(), tok.column())?;

                Ok((tok.clone(), resolved_ty))
            })
            .collect::<Result<_, CompilerError>>()?;

        self.enter_scope();
        for (tok, ty) in &resolved_fields {
            if !self.declare_symbol(
                tok.lexeme(),
                Symbol::Var {
                    name: tok.lexeme().to_string(),
                    tol_type: ty.clone(),
                },
            ) {
                self.exit_scope();
                return Err(self.declared_in_scope_err(tok));
            }
        }
        self.exit_scope();

        // Store fields into the type as a map
        let mut field_map = HashMap::new();
        for (tok, ty) in &resolved_fields {
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

            let field_name = tok.lexeme().to_string();
            field_map.insert(field_name, ty.clone());
        }

        if let Some(type_info) = self.type_table.get_mut(&bagay_type.to_string()) {
            type_info.fields = field_map;
        }

        Ok(())
    }

    fn analyze_itupad(
        &mut self,
        itupad_for: &TolType,
        itupad_block: &Stmt,
        line: usize,
        column: usize,
    ) -> Result<(), CompilerError> {
        self.enter_scope();
        let itupad_for = self.resolve_type(itupad_for, line, column)?;

        let mut analyzed_methods = Vec::new();
        if let Stmt::ItupadBlock { methods, .. } = itupad_block {
            for method in methods {
                if let Stmt::Method {
                    is_static,
                    met_identifier,
                    params,
                    return_type,
                    block,
                    ..
                } = method
                {
                    let analyzed_method = self.analyze_method(
                        &itupad_for,
                        *is_static,
                        met_identifier,
                        params,
                        return_type,
                        block,
                    )?;

                    analyzed_methods.push((analyzed_method, block));
                }
            }
        }

        let type_info = self
            .type_table
            .get_mut(&itupad_for.to_string())
            .ok_or_else(|| {
                CompilerError::new(
                    &format!("Ang tipong {} ay hindi pa na-ideklara", &itupad_for),
                    ErrorKind::Error,
                    line,
                    column,
                )
            })?;

        for (met_sym, _) in &analyzed_methods {
            if let Symbol::Method { name, .. } = met_sym {
                match type_info.methods.entry(name.to_string()) {
                    Entry::Occupied(_) => {
                        return Err(CompilerError::new(
                            &format!("May paraan na na `{}` ang tipong `{}`", name, itupad_for),
                            ErrorKind::Error,
                            line,
                            column,
                        ));
                    }
                    Entry::Vacant(entry) => {
                        entry.insert(met_sym.clone());
                    }
                }
            }
        }

        Ok(())
    }

    fn analyze_method(
        &mut self,
        itupad_for: &TolType,
        is_static: bool,
        met_identifier: &Token,
        params: &[(Token, TolType)],
        return_type: &TolType,
        block: &Expr,
    ) -> Result<Symbol, CompilerError> {
        let resolved_params: Vec<_> = params
            .iter()
            .map(|(tok, ty)| {
                let resolved_type = match ty {
                    TolType::AkoType => itupad_for.clone(),
                    _ => self.resolve_type(ty, tok.line(), tok.column())?,
                };

                Ok((tok.clone(), resolved_type))
            })
            .collect::<Result<_, CompilerError>>()?;

        let param_types = resolved_params.iter().map(|(_, ty)| ty.clone()).collect();
        let symbol = Symbol::Method {
            is_static,
            name: met_identifier.lexeme().to_string(),
            param_types,
            return_type: return_type.clone(),
        };

        if !self.declare_symbol(met_identifier.lexeme(), symbol.clone()) {
            return Err(self.declared_in_scope_err(met_identifier));
        }

        self.enter_scope();
        for (tok, ty) in &resolved_params {
            let param_symbol = Symbol::Var {
                name: tok.lexeme().to_string(),
                tol_type: ty.clone(),
            };

            if !self.declare_symbol(tok.lexeme(), param_symbol) {
                return Err(self.declared_in_scope_err(tok));
            }
        }
        self.current_func_return_type = return_type.clone();
        // println!("{:?}", self.symbol_table);
        self.analyze_expression(block)?;
        self.exit_scope();

        Ok(symbol)
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
                block_value,
                // line,
                // column,
                ..
            } => {
                self.enter_scope();
                for statement in statements {
                    self.analyze_stmt(statement);
                }
                self.exit_scope();

                block_value
                    .as_ref()
                    .map_or_else(|| Ok(TolType::Wala), |expr| self.analyze_expression(expr))
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

                if let Symbol::Paraan {
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

                match self.type_table.get(&left_type.to_string()) {
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
            }
            // TODO: Need to check if arguments are right
            Expr::MethodCall {
                left,
                callee,
                args,
                line,
                column,
            } => {
                let left_type = self.analyze_expression(left)?;
                let mut arg_types = vec![left_type.clone()];
                for arg in args {
                    arg_types.push(self.analyze_expression(arg)?);
                }

                match self.type_table.get(&left_type.to_string()) {
                    Some(type_info) => match type_info.methods.get(callee.lexeme()) {
                        Some(met_sym) => {
                            if let Symbol::Method {
                                param_types,
                                return_type,
                                is_static,
                                ..
                            } = met_sym
                            {
                                if *is_static {
                                    return Err(CompilerError::new(
                                        &format!("{} ay isang static na paraan!", callee.lexeme()),
                                        ErrorKind::Error,
                                        callee.line(),
                                        callee.column(),
                                    ));
                                }

                                if let Err(e) = Self::check_call(&arg_types, param_types) {
                                    return Err(CompilerError::new(
                                        e,
                                        ErrorKind::Error,
                                        *line,
                                        *column,
                                    ));
                                }

                                Ok(return_type.clone())
                            } else {
                                unreachable!("met_sym is not a MetSymbol");
                            }
                        }
                        None => Err(CompilerError::new(
                            &format!(
                                "Walang method na `{}` ang `{}` na tipo",
                                callee.lexeme(),
                                &left_type
                            ),
                            ErrorKind::Error,
                            *line,
                            *column,
                        )),
                    },
                    None => Err(CompilerError::new(
                        &format!("Walang miyembro ang `{}`", &left_type),
                        ErrorKind::Error,
                        *line,
                        *column,
                    )),
                }
            }
            // Expr::StaticFieldAccess {
            //     left,
            //     field,
            //     line,
            //     column,
            // } => {
            //
            // }
            Expr::StaticMethodCall {
                left,
                callee,
                args,
                line,
                column,
            } => {
                let left_type = self.resolve_type(left, callee.line(), callee.column())?;

                let mut arg_types = Vec::new();
                for arg in args {
                    arg_types.push(self.analyze_expression(arg)?);
                }

                match self.type_table.get(&left_type.to_string()) {
                    Some(type_info) => match type_info.methods.get(callee.lexeme()) {
                        Some(met_sym) => {
                            if let Symbol::Method {
                                is_static,
                                param_types,
                                return_type,
                                ..
                            } = met_sym
                            {
                                if !is_static {
                                    return Err(CompilerError::new(
                                        &format!(
                                            "{} ay hindi isang static na paraan",
                                            callee.lexeme()
                                        ),
                                        ErrorKind::Error,
                                        callee.line(),
                                        callee.column(),
                                    ));
                                }

                                if let Err(e) = Self::check_call(&arg_types, param_types) {
                                    return Err(CompilerError::new(
                                        e,
                                        ErrorKind::Error,
                                        *line,
                                        *column,
                                    ));
                                }

                                Ok(return_type.clone())
                            } else {
                                unreachable!("met_sym is not a MetSymbol");
                            }
                        }
                        None => Err(CompilerError::new(
                            &format!(
                                "Walang method na `{}` ang `{}` na tipo",
                                callee.lexeme(),
                                &left_type
                            ),
                            ErrorKind::Error,
                            *line,
                            *column,
                        )),
                    },
                    None => Err(CompilerError::new(
                        &format!("Hindi rehistrado ang tipong {}", &left_type),
                        ErrorKind::Error,
                        *line,
                        *column,
                    )),
                }
            }
            Expr::Struct {
                name,
                fields,
                line,
                column,
            } => {
                let resolved_type = self.resolve_type(name, *line, *column)?;

                let type_info = match self.type_table.get(&resolved_type.to_string()) {
                    Some(t_info) => t_info.clone(),
                    None => {
                        return Err(CompilerError::new(
                            &format!("Hindi rehistrado ang tipo na `{}`", &resolved_type),
                            ErrorKind::Error,
                            *line,
                            *column,
                        ));
                    }
                };

                if fields.len() != type_info.fields.len() {
                    return Err(CompilerError::new(
                        &format!(
                            "Hindi wastong bilang ng fields, ang {} ay may {} na bilang ng fields",
                            &resolved_type,
                            type_info.fields.len()
                        ),
                        ErrorKind::Error,
                        *line,
                        *column,
                    ));
                }

                for (field_tok, field_expr) in fields {
                    let field_name = field_tok.lexeme();
                    let field_type = self.analyze_expression(field_expr)?;

                    match type_info.fields.get(field_name) {
                        Some(t) => {
                            if !field_type.is_assignment_compatible(t) {
                                return Err(CompilerError::new(
                                    &format!("Hindi pwede ilagay ang {} sa {}", field_type, t),
                                    ErrorKind::Error,
                                    field_tok.line(),
                                    field_tok.column(),
                                ));
                            }
                        }
                        None => {
                            return Err(CompilerError::new(
                                &format!(
                                    "Ang field na {} ay wala sa {}",
                                    field_name, resolved_type
                                ),
                                ErrorKind::Error,
                                field_tok.line(),
                                field_tok.column(),
                            ));
                        }
                    }
                }

                Ok(resolved_type)
            }
            _ => Ok(TolType::Wala),
        }
    }

    fn check_call(args: &[TolType], params: &[TolType]) -> Result<(), &'static str> {
        if args.len() != params.len() {
            return Err("Ang bilang ng argumento ay hindi pareho sa parameter");
        }

        if !args
            .iter()
            .zip(params.iter())
            .all(|(arg, param)| arg.is_assignment_compatible(param))
        {
            return Err("Hindi wastong tipo ang argumento para sa parameter");
        }

        Ok(())
    }

    fn declare_magic_funcs(&mut self) {
        let magic_symbols = vec![
            (
                "print",
                Symbol::Paraan {
                    name: "print".to_string(),
                    param_types: vec![TolType::Sinulid],
                    return_type: TolType::Wala,
                },
            ),
            (
                "println",
                Symbol::Paraan {
                    name: "println".to_string(),
                    param_types: vec![TolType::Sinulid],
                    return_type: TolType::Wala,
                },
            ),
            (
                "alis",
                Symbol::Paraan {
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

    pub fn has_error(&self) -> bool {
        self.has_error
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
