use std::collections::{HashMap, hash_map::Entry};

use crate::{
    error::{CompilerError, ErrorKind},
    lexer::{token::Token, token_kind::TokenKind},
    parser::ast::{
        expr::Expr,
        stmt::{KungBranch, Stmt},
    },
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
    inferred_types: HashMap<usize, TolType>,
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
            inferred_types: HashMap::new(),
        };

        // Declare magic functions first
        new_analyzer.declare_magic_funcs();
        new_analyzer.declare_primitive_types();

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
                ..
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

        // for (id, ty) in &self.inferred_types {
        //     println!("{} => {}", id, ty);
        // }

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
                id,
                mutable,
            } => self
                .analyze_ang(*mutable, ang_identifier, ang_type, rhs, *line, *column, *id)
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
            Stmt::Ibalik {
                rhs, line, column, ..
            } => self.analyze_ibalik(rhs, line, column).unwrap_or_else(|e| {
                self.has_error = true;
                e.display(self.source_path);
            }),
            Stmt::ExprS { expr, .. } => {
                if let Err(e) = self.analyze_expression(expr) {
                    self.has_error = true;
                    e.display(self.source_path);
                }
            }
            Stmt::Bagay {
                bagay_identifier,
                fields,
                ..
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
                ..
            } => {
                if let Err(e) = self.analyze_itupad(itupad_for, itupad_block, *line, *column) {
                    self.has_error = true;
                    e.display(self.source_path);
                }
            }
            Stmt::Kung { branches, .. } => self.analyze_kung(branches).unwrap_or_else(|e| {
                self.has_error = true;
                e.display(self.source_path);
            }),
            Stmt::Sa {
                iterator,
                bind,
                block,
                id,
                ..
            } => self
                .analyze_sa(iterator, bind, block, *id)
                .unwrap_or_else(|e| {
                    self.has_error = true;
                    e.display(self.source_path);
                }),
            // TODO: analyze ts
            Stmt::ItupadBlock { .. } => {}
            Stmt::Method { .. } => {}
            Stmt::Program(_) => {}
        };
    }

    #[allow(clippy::too_many_arguments)]
    // FIXME: Too many arguments. Individual analyze
    // functions for each statement needs a fix like passing
    // the Stmt enum instead
    fn analyze_ang(
        &mut self,
        mutable: bool,
        ang_identifier: &Token,
        ang_type: &TolType,
        rhs: &Expr,
        line: usize,
        column: usize,
        id: usize,
    ) -> Result<(), CompilerError> {
        let ang_type = match ang_type {
            TolType::Unknown => self.infer_type(rhs, id)?,
            _ => self.resolve_type(ang_type, line, column)?,
        };

        let rhs_type = self.analyze_expression(rhs)?;
        // println!("{:?}, {:?}", rhs_type, ang_type);
        rhs_type.is_assignment_compatible(&ang_type, line, column)?;

        let var_symbol = Symbol::Var {
            mutable,
            name: ang_identifier.lexeme().to_string(),
            tol_type: ang_type.clone(),
        };

        if !self.declare_symbol(ang_identifier.lexeme(), var_symbol) {
            return Err(self.declared_in_scope_err(ang_identifier));
        }

        Ok(())
    }

    fn infer_type(&mut self, expr: &Expr, id: usize) -> Result<TolType, CompilerError> {
        let expr_type = self.analyze_expression(expr)?;

        let inferred_type = self.resolve_expr_type(expr_type);

        self.inferred_types.insert(id, inferred_type.clone());

        Ok(inferred_type)
    }

    #[allow(clippy::only_used_in_recursion)]
    fn resolve_expr_type(&self, type_: TolType) -> TolType {
        match type_ {
            TolType::UnsizedInt => TolType::I32,
            TolType::UnsizedFloat => TolType::DobleTang,
            TolType::Array(t, len) => {
                TolType::Array(Box::new(self.resolve_expr_type(*t).clone()), len)
            }
            _ => type_,
        }
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
            // TODO: Add mutability on parameters later
            let param_symbol = Symbol::Var {
                mutable: false,
                name: tok.lexeme().to_string(),
                tol_type: ty.clone(),
            };

            if !self.declare_symbol(tok.lexeme(), param_symbol) {
                return Err(self.declared_in_scope_err(tok));
            }
        }
        self.current_func_return_type = resolved_return.clone();
        let last_expr_type = self.analyze_expression(block)?;
        last_expr_type.is_assignment_compatible(
            &self.current_func_return_type,
            par_identifier.line(),
            par_identifier.column(),
        )?;
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

        return_type.is_assignment_compatible(&self.current_func_return_type, *line, *column)?;

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
                // TODO: Add mutability on fields later
                Symbol::Var {
                    mutable: false,
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
            // TODO: Add mutability on params later
            let param_symbol = Symbol::Var {
                mutable: false,
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

    fn analyze_kung(&mut self, branches: &[KungBranch]) -> Result<(), CompilerError> {
        for branch in branches {
            if let Some(s) = &branch.condition {
                self.analyze_expression(s)?;
            }

            self.analyze_expression(&branch.block)?;
        }

        Ok(())
    }

    fn analyze_sa(
        &mut self,
        iterator: &Expr,
        bind: &Token,
        block: &Expr,
        id: usize,
    ) -> Result<(), CompilerError> {
        self.enter_scope();
        // println!("{}", id);
        let bind_type = self.infer_type(iterator, id)?;
        let bind_symbol = Symbol::Var {
            mutable: false,
            name: bind.lexeme().to_string(),
            tol_type: bind_type,
        };

        if !self.declare_symbol(bind.lexeme(), bind_symbol) {
            return Err(self.declared_in_scope_err(bind));
        }
        self.analyze_expression(block)?;
        self.exit_scope();

        Ok(())
    }

    fn analyze_expression(&mut self, expr: &Expr) -> Result<TolType, CompilerError> {
        match expr {
            Expr::IntLit { .. } => Ok(TolType::UnsizedInt),
            Expr::FloatLit { .. } => Ok(TolType::UnsizedFloat),
            // Expr::StringLit { .. } => Ok(TolType::Sinulid),
            Expr::ByteStringLit { token, .. } => Ok(TolType::Array(
                Box::new(TolType::U8),
                Some(token.lexeme().len() + 1),
            )),
            Expr::Identifier { token, .. } => Ok(self
                .lookup_symbol(token.lexeme(), token.line(), token.column())?
                .get_type()
                .clone()),
            Expr::Binary {
                op, left, right, ..
            } => match op.kind() {
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
            Expr::Assign {
                left,
                right,
                line,
                column,
                ..
            } => {
                if !left.is_lvalue() {
                    return Err(CompilerError::new(
                        "Ang nasa kaliwa ng `=` ay hindi isang lvalue",
                        ErrorKind::Error,
                        *line,
                        *column,
                    ));
                }

                self.ensure_mutability(left)?;

                let left_type = self.analyze_expression(left)?;
                let right_type = self.analyze_expression(right)?;

                right_type.is_assignment_compatible(&left_type, *line, *column)?;

                Ok(TolType::Wala)
            }
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
            Expr::FnCall { callee, args, .. } => {
                let symbol = self.lookup_symbol(callee.lexeme(), callee.line(), callee.column())?;

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
                        arg.is_assignment_compatible(param, callee.line(), callee.column())?;
                    }

                    return Ok(return_type_);
                }
                panic!("The symbol is not declared as `par` symbol");
            }
            Expr::MagicFnCall { fncall, .. } => self.analyze_expression(fncall),
            Expr::FieldAccess {
                left,
                member,
                line,
                column,
                ..
            } => {
                let left_type = self.analyze_expression(left)?;

                let type_info = self.lookup_type(&left_type, *line, *column)?;
                match type_info.fields.get(member.lexeme()) {
                    Some(toltype) => Ok(toltype.to_owned()),
                    None => Err(CompilerError::new(
                        &format!(
                            "Ang `{}` ay hindi kabilang sa `{}` na tipo",
                            member.lexeme(),
                            &left_type
                        ),
                        ErrorKind::Error,
                        member.line(),
                        member.column(),
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
                ..
            } => {
                let left_type = self.analyze_expression(left)?;
                let mut arg_types = vec![left_type.clone()];
                for arg in args {
                    arg_types.push(self.analyze_expression(arg)?);
                }

                let type_info = self.lookup_type(&left_type, *line, *column)?;
                match type_info.methods.get(callee.lexeme()) {
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

                            Self::check_call(&arg_types, param_types, *line, *column)?;

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
                ..
            } => {
                let left_type = self.resolve_type(left, callee.line(), callee.column())?;

                let mut arg_types = Vec::new();
                for arg in args {
                    arg_types.push(self.analyze_expression(arg)?);
                }

                let type_info = self.lookup_type(&left_type, *line, *column)?;
                match type_info.methods.get(callee.lexeme()) {
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
                                    &format!("{} ay hindi isang static na paraan", callee.lexeme()),
                                    ErrorKind::Error,
                                    callee.line(),
                                    callee.column(),
                                ));
                            }

                            Self::check_call(&arg_types, param_types, *line, *column)?;

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
                }
            }
            Expr::Struct {
                name,
                fields,
                line,
                column,
                ..
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
                            field_type.is_assignment_compatible(
                                t,
                                field_tok.line(),
                                field_tok.column(),
                            )?;
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
            Expr::Array {
                elements,
                line,
                column,
                ..
            } => {
                let assumed_element_type = self.analyze_expression(&elements[0])?;

                if elements.len() > 1 {
                    for elem in elements[1..elements.len() - 1].iter() {
                        let elem_type = self.analyze_expression(elem)?;
                        elem_type.is_assignment_compatible(
                            &assumed_element_type,
                            *line,
                            *column,
                        )?;
                    }
                }

                Ok(TolType::Array(
                    Box::new(assumed_element_type),
                    Some(elements.len()),
                ))
            }
            Expr::RangeExclusive {
                start,
                end,
                line,
                column,
                ..
            }
            | Expr::RangeInclusive {
                start,
                end,
                line,
                column,
                ..
            } => {
                let left_type = self.analyze_expression(start)?;
                let right_type = self.analyze_expression(end)?;

                if let Err(e) = left_type.is_assignment_compatible(&TolType::USukat, *line, *column)
                {
                    return Err(e.add_note(
                        "Dapat ang simula at wakas ng `..` na operasyon ay may usukat na tipo",
                    ));
                };
                if let Err(e) =
                    right_type.is_assignment_compatible(&TolType::USukat, *line, *column)
                {
                    return Err(e.add_note(
                        "Dapat ang simula at wakas ng `..` na operasyon ay may usukat na tipo",
                    ));
                };

                Ok(TolType::USukat)
            }
            _ => Ok(TolType::Wala),
        }
    }

    fn check_call(
        args: &[TolType],
        params: &[TolType],
        line: usize,
        column: usize,
    ) -> Result<(), CompilerError> {
        if args.len() != params.len() {
            return Err(CompilerError::new(
                "Ang bilang ng argumento ay hindi pareho sa parameter",
                ErrorKind::Error,
                line,
                column,
            ));
        }

        for (arg, param) in args.iter().zip(params) {
            arg.is_assignment_compatible(param, line, column)?;
        }

        Ok(())
    }

    fn declare_magic_funcs(&mut self) {
        let magic_symbols = vec![
            (
                "print",
                Symbol::Paraan {
                    name: "print".to_string(),
                    param_types: vec![TolType::Array(Box::new(TolType::U8), None)],
                    return_type: TolType::Wala,
                },
            ),
            (
                "println",
                Symbol::Paraan {
                    name: "println".to_string(),
                    param_types: vec![TolType::Array(Box::new(TolType::U8), None)],
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

    fn declare_primitive_types(&mut self) {
        self.type_table
            .insert("i32".to_string(), TypeInfo::new(TolType::I32));
    }

    fn lookup_symbol(
        &self,
        name: &str,
        line: usize,
        column: usize,
    ) -> Result<&Symbol, CompilerError> {
        for scope in self.symbol_table.iter().rev() {
            if let Some(s) = scope.get(name) {
                return Ok(s);
            }
        }

        Err(CompilerError::new(
            &format!("Ang `{}` ay hindi na-ideklara", name),
            ErrorKind::Error,
            line,
            column,
        ))
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

    fn lookup_type(
        &self,
        type_: &TolType,
        line: usize,
        column: usize,
    ) -> Result<&TypeInfo, CompilerError> {
        self.type_table
            .get(&type_.to_string())
            .ok_or(CompilerError::new(
                &format!("Ang `{}` ay hindi naideklarang tipo", type_),
                ErrorKind::Error,
                line,
                column,
            ))
    }

    fn ensure_mutability(&self, lvalue: &Expr) -> Result<(), CompilerError> {
        // WARN: Only works for identifiers for now
        if let Expr::Identifier { token, .. } = lvalue
            && let Symbol::Var { mutable, .. } =
                self.lookup_symbol(token.lexeme(), token.line(), token.column())?
            && !*mutable
        {
            return Err(CompilerError::new(
                &format!("Ang `{}` ay hindi `maiba`", token.lexeme()),
                ErrorKind::Error,
                token.line(),
                token.column(),
            )
            .add_help("Subukan mong lagyan ng `maiba` ang deklarasyon nito"));
        }

        Ok(())
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

    pub fn inferred_types(&self) -> &HashMap<usize, TolType> {
        &self.inferred_types
    }
}
