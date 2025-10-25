use std::collections::{HashMap, hash_map::Entry};

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
    symbol::Symbol,
    toltype::{TolType, type_info::TypeInfo},
};

pub struct SemanticAnalyzer<'a> {
    parent_module: &'a mut Module,
    has_error: bool,
    current_func_return_type: TolType,
}

impl<'a> SemanticAnalyzer<'a> {
    pub fn new(parent_module: &'a mut Module) -> Self {
        let mut new_analyzer = Self {
            parent_module,
            has_error: false,
            current_func_return_type: TolType::Unknown,
        };

        // Declare magic functions first
        new_analyzer.declare_magic_funcs();

        // Declare types
        new_analyzer.declare_primitive_types();

        new_analyzer
    }

    fn declare_primitive_types(&mut self) {
        // Signed integers
        let type_table = &mut self.parent_module.type_table;

        type_table.insert("i8".to_string(), TypeInfo::new(TolType::I8));
        type_table.insert("i16".to_string(), TypeInfo::new(TolType::I16));
        type_table.insert("i32".to_string(), TypeInfo::new(TolType::I32));
        type_table.insert("i64".to_string(), TypeInfo::new(TolType::I64));
        type_table.insert("isukat".to_string(), TypeInfo::new(TolType::ISukat));

        // Unsigned integers
        type_table.insert("u8".to_string(), TypeInfo::new(TolType::U8));
        type_table.insert("u16".to_string(), TypeInfo::new(TolType::U16));
        type_table.insert("u32".to_string(), TypeInfo::new(TolType::U32));
        type_table.insert("u64".to_string(), TypeInfo::new(TolType::U64));
        type_table.insert("usukat".to_string(), TypeInfo::new(TolType::USukat));

        // Floating-point
        type_table.insert("lutang".to_string(), TypeInfo::new(TolType::Lutang));
        type_table.insert("dobletang".to_string(), TypeInfo::new(TolType::DobleTang));

        // Others
        type_table.insert("bool".to_string(), TypeInfo::new(TolType::Bool));
        type_table.insert("kar".to_string(), TypeInfo::new(TolType::Kar));
    }

    pub fn analyze(&mut self) {
        // Temporarily own statements
        let statements = std::mem::take(&mut self.parent_module.ast);

        // Collect type declarations
        for stmt in &statements {
            if let Stmt::Bagay {
                bagay_identifier,
                fields,
                ..
            } = stmt
            {
                self.analyze_bagay(bagay_identifier, fields)
                    .unwrap_or_else(|e| {
                        self.has_error = true;
                        e.display(&self.parent_module.source_path)
                    });
            }
        }

        // Second pass: analyze everything else
        for stmt in &statements {
            if !matches!(stmt, Stmt::Bagay { .. }) {
                self.analyze_stmt(stmt).unwrap_or_else(|e| {
                    e.display(&self.parent_module.source_path);
                    self.has_error = true;
                });
            }
        }

        // Put it back
        self.parent_module.ast = statements;

        // for (id, ty) in &self.inferred_types {
        //     println!("{} => {}", id, ty);
        // }

        // println!("{:?}", self.type_table.keys());
        // println!("{:#?}", self.type_table);
    }

    fn analyze_stmt(&mut self, stmt: &Stmt) -> Result<(), CompilerError> {
        match stmt {
            Stmt::Ang {
                ang_identifier,
                ang_type,
                rhs,
                line,
                column,
                id,
                mutable,
            } => self.analyze_ang(*mutable, ang_identifier, ang_type, rhs, *line, *column, *id),
            Stmt::Par {
                par_identifier,
                params,
                return_type,
                block,
                // line,
                // column,
                ..
            } => self.analyze_par(par_identifier, params, return_type, block),
            Stmt::Ibalik {
                rhs, line, column, ..
            } => self.analyze_ibalik(rhs, line, column),
            Stmt::ExprS { expr, .. } => self.analyze_expression(expr).map(|_| ()),
            Stmt::Bagay {
                bagay_identifier,
                fields,
                ..
            } => self.analyze_bagay(bagay_identifier, fields),
            Stmt::Itupad {
                itupad_for,
                itupad_block,
                line,
                column,
                ..
            } => self.analyze_itupad(itupad_for, itupad_block, *line, *column),

            Stmt::Kung { branches, .. } => self.analyze_kung(branches),
            Stmt::Sa {
                iterator,
                bind,
                block,
                id,
                ..
            } => self.analyze_sa(iterator, bind, block, *id),
            Stmt::Block { statements, .. } => self.analyze_block(statements),
            // TODO: analyze ts
            Stmt::ItupadBlock { .. } => Ok(()),
            Stmt::Method { .. } => Ok(()),
        }
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

        self.declare_array_types(&ang_type);

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

    fn declare_array_types(&mut self, array_type: &TolType) {
        if let TolType::Array(inner, _) = array_type {
            // Step 1: recurse on inner arrays first
            self.declare_array_types(inner);

            // Step 2: get the element type (inner type) C name
            let inner_c = inner.as_c(); // e.g., "int32_t" or "TOL_Array_int32_t"
            let array_c = format!("TOL_Array_{}", inner_c);

            // Step 3: store this array type if not already declared
            if !self.parent_module.declared_array_types.contains(&array_c) {
                self.parent_module.declared_array_types.push(array_c);
            }
        }
    }

    pub fn infer_type(&mut self, expr: &Expr, id: usize) -> Result<TolType, CompilerError> {
        let expr_type = self.analyze_expression(expr)?;

        let inferred_type = self.resolve_expr_type(expr_type);

        self.parent_module
            .inferred_types
            .insert(id, inferred_type.clone());

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
        &mut self,
        type_to_resolve: &TolType,
        line: usize,
        column: usize,
    ) -> Result<TolType, CompilerError> {
        let type_table = &mut self.parent_module.type_table;
        match type_to_resolve {
            TolType::UnknownIdentifier(id) => {
                type_table
                    .get(id)
                    .map(|t| t.kind.clone())
                    .ok_or(CompilerError::new(
                        &format!("Hindi valid na tipo ang `{}`", id),
                        ErrorKind::Error,
                        line,
                        column,
                    ))
            }
            _ => Ok(type_to_resolve.clone()),
        }
    }

    fn analyze_par(
        &mut self,
        par_identifier: &Token,
        params: &[(Token, TolType)],
        return_type: &TolType,
        block: &Stmt,
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
        self.analyze_stmt(block)?;
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
        match self.parent_module.type_table.entry(bagay_name.to_string()) {
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
                    members: HashMap::new(),
                    static_members: HashMap::new(),
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
        let field_map = &mut self
            .parent_module
            .type_table
            .get_mut(&bagay_type.to_string())
            .unwrap()
            .members;
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
            field_map.insert(
                field_name.clone(),
                Symbol::Var {
                    name: field_name,
                    mutable: false,
                    tol_type: ty.clone(),
                },
            );
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
            .parent_module
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
            if let Symbol::Method {
                name, is_static, ..
            } = met_sym
            {
                let mut insert_to = &mut type_info.members;
                if *is_static {
                    insert_to = &mut type_info.static_members;
                }
                match insert_to.entry(name.to_string()) {
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
        block: &Stmt,
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
        self.analyze_stmt(block)?;
        self.exit_scope();

        Ok(symbol)
    }

    fn analyze_kung(&mut self, branches: &[KungBranch]) -> Result<(), CompilerError> {
        for branch in branches {
            if let Some(s) = &branch.condition {
                self.analyze_expression(s)?;
            }

            self.analyze_stmt(&branch.block)?;
        }

        Ok(())
    }

    fn analyze_sa(
        &mut self,
        iterator: &Expr,
        bind: &Token,
        block: &Stmt,
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
        self.analyze_stmt(block)?;
        self.exit_scope();

        Ok(())
    }

    fn analyze_block(&mut self, statements: &[Stmt]) -> Result<(), CompilerError> {
        self.enter_scope();

        for statement in statements {
            self.analyze_stmt(statement)?;
        }

        self.exit_scope();
        Ok(())
    }

    pub fn analyze_expression(&mut self, expr: &Expr) -> Result<TolType, CompilerError> {
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

                self.ensure_lvalue_is_mutable(left, *line, *column)?;

                let left_type = self.analyze_expression(left)?;
                let right_type = self.analyze_expression(right)?;

                right_type.is_assignment_compatible(&left_type, *line, *column)?;

                Ok(TolType::Wala)
            }
            Expr::FnCall { line, column, .. } => self.analyze_fncall(expr, *line, *column),
            Expr::MagicFnCall { name, args, .. } => {
                let (line, column) = (name.line(), name.column());
                let arg_types: Vec<TolType> = args
                    .iter()
                    .map(|arg| self.analyze_expression(arg))
                    .collect::<Result<_, CompilerError>>()?;

                let sym = self.lookup_symbol(name.lexeme(), line, column)?;
                match sym {
                    Symbol::Paraan {
                        param_types,
                        return_type,
                        ..
                    } => {
                        Self::check_call(&arg_types, param_types, line, column)?;

                        Ok(return_type.clone())
                    }
                    _ => Err(CompilerError::new(
                        &format!("Hindi nahanap ang `{}`", name.lexeme()),
                        ErrorKind::Error,
                        line,
                        column,
                    )),
                }
            }
            Expr::MemberAccess { .. } => self.analyze_member_access(expr),
            Expr::ScopeResolution { .. } => self.analyze_scope_resolution(expr),
            Expr::Struct { .. } => self.analyze_struct_expr(expr),
            Expr::Array {
                elements,
                line,
                column,
                id,
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

                let resulting_type =
                    TolType::Array(Box::new(assumed_element_type), Some(elements.len()));

                self.parent_module
                    .inferred_types
                    .insert(*id, resulting_type.clone());

                Ok(resulting_type)
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
            Expr::AddressOf {
                of, line, column, ..
            } => {
                if !of.is_lvalue() {
                    return Err(CompilerError::new(
                        "Hindi pwedeng rvalue ang nasa kanan ng `&`",
                        ErrorKind::Error,
                        *line,
                        *column,
                    )
                        .add_note("Ang rvalue ay tumutukoy sa mga expresyon na makikita mo sa kanan ng assignment (`=`) pero hindi mo makikita sa kaliwa")
                        .add_note("Ang halimbawa nito ay mga integer na literal (41, 67) o mga pagtawag ng paraan (tawag())"));
                }

                // of can be mutable or immutable, so we are not gonna check it

                let of_type = self.analyze_expression(of)?;

                Ok(TolType::Pointer(Box::new(of_type)))
            }
            Expr::MutableAddressOf {
                of, line, column, ..
            } => {
                if !of.is_lvalue() {
                    return Err(CompilerError::new(
                        "Hindi pwedeng rvalue ang nasa kanan ng `&`",
                        ErrorKind::Error,
                        *line,
                        *column,
                    )
                        .add_note("Ang rvalue ay tumutukoy sa mga expresyon na makikita mo sa kanan ng assignment (`=`) pero hindi mo makikita sa kaliwa")
                        .add_note("Ang halimbawa nito ay mga integer na literal (41, 67) o mga pagtawag ng paraan (tawag())"));
                }

                self.ensure_lvalue_is_mutable(of, *line, *column)?;

                let of_type = self.analyze_expression(of)?;

                Ok(TolType::MutablePointer(Box::new(of_type)))
            }
            Expr::Deref {
                right,
                line,
                column,
                ..
            } => {
                if !right.is_lvalue() {
                    return Err(CompilerError::new(
                        "Hindi pwedeng rvalue ang nasa kanan ng `*`",
                        ErrorKind::Error,
                        *line,
                        *column,
                    )
                        .add_note("Ang rvalue ay tumutukoy sa mga expresyon na makikita mo sa kanan ng assignment (`=`) pero hindi mo makikita sa kaliwa")
                        .add_note("Ang halimbawa nito ay mga integer na literal (41, 67) o mga pagtawag ng paraan (tawag())"));
                }
                let right_type = self.analyze_expression(right)?;

                match &right_type {
                    TolType::Pointer(t) => Ok(t.as_ref().clone()),
                    _ => Err(CompilerError::new(
                        &format!(
                            "Ang nasa kanan ng `*` ay hindi isang pointer, kundi ito ay `{}`",
                            right_type
                        ),
                        ErrorKind::Error,
                        *line,
                        *column,
                    )),
                }
            }
            Expr::StringLit { .. } => todo!(),
        }
    }

    fn analyze_fncall(
        &mut self,
        fncall: &Expr,
        line: usize,
        column: usize,
    ) -> Result<TolType, CompilerError> {
        if let Expr::FnCall { callee, args, .. } = fncall {
            let mut arg_types: Vec<TolType> = args
                .iter()
                .map(|arg| self.analyze_expression(arg))
                .collect::<Result<_, CompilerError>>()?;
            if let Expr::MemberAccess { left, .. } = callee.as_ref() {
                let left_type = self.analyze_expression(left)?;
                arg_types.insert(0, left_type);
            }

            let callee_symbol = self.lookup_lvalue(callee, line, column)?;

            match callee_symbol {
                Symbol::Paraan {
                    param_types,
                    return_type,
                    ..
                } => {
                    Self::check_call(&arg_types, param_types, line, column)?;

                    Ok(return_type.clone())
                }
                Symbol::Method {
                    param_types,
                    return_type,
                    ..
                } => {
                    Self::check_call(&arg_types, param_types, line, column)?;

                    Ok(return_type.clone())
                }
                _ => Err(CompilerError::new(
                    "Invalid ang tigatawag ng paraan",
                    ErrorKind::Error,
                    line,
                    column,
                )),
            }
        } else {
            unreachable!()
        }
    }

    fn analyze_member_access(&mut self, expr: &Expr) -> Result<TolType, CompilerError> {
        if let Expr::MemberAccess {
            left,
            member,
            line,
            column,
            ..
        } = expr
        {
            let sym = self.lookup_member_access(left, member, *line, *column)?;
            match sym {
                Symbol::Var { tol_type, .. } => Ok(tol_type.clone()),
                Symbol::Paraan { return_type, .. } | Symbol::Method { return_type, .. } => {
                    Ok(return_type.clone())
                }
                Symbol::Bagay { name } => Ok(TolType::Bagay(name.clone())),
            }
        } else {
            unreachable!()
        }
    }

    fn analyze_struct_expr(&mut self, struct_expr: &Expr) -> Result<TolType, CompilerError> {
        if let Expr::Struct {
            callee,
            fields,
            line,
            column,
            ..
        } = struct_expr
        {
            let resolved_fields: Vec<(Token, TolType)> = fields
                .iter()
                .map(|(tok, ex)| {
                    let ex_type = self.analyze_expression(ex)?;
                    Ok((tok.clone(), ex_type))
                })
                .collect::<Result<_, CompilerError>>()?;

            let callee_symbol = match callee.as_ref() {
                Expr::Identifier { token, .. } => {
                    self.lookup_symbol(token.lexeme(), token.line(), token.column())
                }
                Expr::MemberAccess {
                    left,
                    member,
                    line,
                    column,
                    ..
                } => self.lookup_member_access(left, member, *line, *column),
                Expr::ScopeResolution {
                    left,
                    field,
                    line,
                    column,
                    ..
                } => self.lookup_scope_resolution(left, field, *line, *column),
                _ => Err(CompilerError::new(
                    "Hindi ito pwede i-construct",
                    ErrorKind::Error,
                    *line,
                    *column,
                )),
            }?;

            let bagay_name: String;
            let members = match callee_symbol.clone() {
                Symbol::Bagay { name } => {
                    bagay_name = name.clone();
                    &self.parent_module.type_table.get(&name).unwrap().members
                }
                _ => {
                    return Err(CompilerError::new(
                        "Hindi pwede i-construct ang hindi bagay",
                        ErrorKind::Error,
                        *line,
                        *column,
                    ));
                }
            };

            for (field_tok, field_ty) in &resolved_fields {
                let field_symbol = members.get(field_tok.lexeme()).ok_or(CompilerError::new(
                    &format!(
                        "Walang field na `{}` ang `{}`",
                        field_tok.lexeme(),
                        bagay_name
                    ),
                    ErrorKind::Error,
                    field_tok.line(),
                    field_tok.column(),
                ))?;

                match field_symbol {
                    Symbol::Var { tol_type, .. } => {
                        field_ty.is_assignment_compatible(
                            tol_type,
                            field_tok.line(),
                            field_tok.column(),
                        )?;
                    }
                    _ => {
                        return Err(CompilerError::new(
                            &format!("Hindi field ang `{}`", field_tok.lexeme()),
                            ErrorKind::Error,
                            field_tok.line(),
                            field_tok.column(),
                        ));
                    }
                }
            }

            Ok(TolType::Bagay(bagay_name.clone()))
        } else {
            unreachable!()
        }
    }

    fn analyze_scope_resolution(&self, expr: &Expr) -> Result<TolType, CompilerError> {
        if let Expr::ScopeResolution {
            left,
            field,
            line,
            column,
            ..
        } = expr
        {
            let sym = self.lookup_scope_resolution(left, field, *line, *column)?;

            match sym {
                Symbol::Var { tol_type, .. } => Ok(tol_type.clone()),
                Symbol::Paraan { return_type, .. } | Symbol::Method { return_type, .. } => {
                    Ok(return_type.clone())
                }
                Symbol::Bagay { name } => Ok(TolType::Bagay(name.clone())),
            }
        } else {
            unreachable!()
        }
    }

    fn lookup_lvalue(
        &mut self,
        lvalue: &Expr,
        line: usize,
        column: usize,
    ) -> Result<&Symbol, CompilerError> {
        match lvalue {
            Expr::Identifier { token, .. } => {
                self.lookup_symbol(token.lexeme(), token.line(), token.column())
            }
            Expr::MemberAccess {
                left,
                member,
                line,
                column,
                ..
            } => self.lookup_member_access(left, member, *line, *column),
            Expr::ScopeResolution {
                left,
                field,
                line,
                column,
                ..
            } => self.lookup_scope_resolution(left, field, *line, *column),
            _ => Err(CompilerError::new(
                "Hindi ito pwedeng tawagin",
                ErrorKind::Error,
                line,
                column,
            )),
        }
    }

    fn lookup_member_access(
        &mut self,
        left: &Expr,
        member: &Token,
        line: usize,
        column: usize,
    ) -> Result<&Symbol, CompilerError> {
        let left_type = self.analyze_expression(left)?;

        let type_info = self
            .parent_module
            .type_table
            .get(&left_type.to_string())
            .ok_or(CompilerError::new(
                &format!("Hindi nahanap ang tipong `{}` sa type table", left_type),
                ErrorKind::Error,
                line,
                column,
            ))?;

        type_info
            .members
            .get(member.lexeme())
            .ok_or(CompilerError::new(
                &format!(
                    "Walang miyembro na `{}` ang `{}`",
                    member.lexeme(),
                    left_type
                ),
                ErrorKind::Error,
                line,
                column,
            ))
    }

    fn lookup_scope_resolution(
        &self,
        left: &Expr,
        field: &Token,
        line: usize,
        column: usize,
    ) -> Result<&Symbol, CompilerError> {
        match left {
            Expr::Identifier { token, .. } => {
                let type_info =
                    self.parent_module
                        .type_table
                        .get(token.lexeme())
                        .ok_or(CompilerError::new(
                            &format!("Hindi narehistro ang `{}`", token.lexeme()),
                            ErrorKind::Error,
                            line,
                            column,
                        ))?;

                type_info
                    .static_members
                    .get(field.lexeme())
                    .ok_or(CompilerError::new(
                        &format!(
                            "Walang static na miyembro na `{}` ang `{}`",
                            field.lexeme(),
                            token.lexeme()
                        ),
                        ErrorKind::Error,
                        field.line(),
                        field.column(),
                    ))
            }
            Expr::ScopeResolution {
                left,
                field,
                line,
                column,
                ..
            } => self.lookup_scope_resolution(left, field, *line, *column),
            _ => Err(CompilerError::new(
                "Hindi valid ang nasa kaliwa ng `::",
                ErrorKind::Error,
                line,
                column,
            )),
        }
    }

    fn check_call(
        args: &[TolType],
        params: &[TolType],
        line: usize,
        column: usize,
    ) -> Result<(), CompilerError> {
        // println!("{:?}\n{:?}", args, params);
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

    fn lookup_symbol(
        &self,
        name: &str,
        line: usize,
        column: usize,
    ) -> Result<&Symbol, CompilerError> {
        for scope in self.parent_module.symbol_table.iter().rev() {
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
        let current_scope = self.parent_module.symbol_table.last_mut().unwrap();

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

    // fn lookup_type(
    //     &self,
    //     type_: &TolType,
    //     line: usize,
    //     column: usize,
    // ) -> Result<&TypeInfo, CompilerError> {
    //     self.type_table
    //         .get(&type_.to_string())
    //         .ok_or(CompilerError::new(
    //             &format!("Ang `{}` ay hindi naideklarang tipo", type_),
    //             ErrorKind::Error,
    //             line,
    //             column,
    //         ))
    // }
    fn ensure_lvalue_is_mutable(
        &mut self,
        lvalue: &Expr,
        line: usize,
        column: usize,
    ) -> Result<(), CompilerError> {
        // // WARN: Only works for identifiers for now
        // if let Expr::Identifier { token, .. } = lvalue
        //     && let Symbol::Var { mutable, .. } =
        //         self.lookup_symbol(token.lexeme(), token.line(), token.column())?
        //     && !*mutable
        // {
        //     return Err(CompilerError::new(
        //         &format!("Ang `{}` ay hindi `maiba`", token.lexeme()),
        //         ErrorKind::Error,
        //         token.line(),
        //         token.column(),
        //     )
        //     .add_help("Subukan mong lagyan ng `maiba` ang deklarasyon nito"));
        // }
        let lvalue_symbol = self.lookup_lvalue(lvalue, line, column)?;

        match lvalue_symbol {
            Symbol::Var { name, mutable, .. } => {
                if *mutable {
                    Ok(())
                } else {
                    Err(CompilerError::new(
                        &format!("Ang `{}` ay hindi `maiba`", name),
                        ErrorKind::Error,
                        line,
                        column,
                    )
                    .add_help("Subukan mong lagyan ng `maiba` ang deklarasyon nito"))
                }
            }
            // WARN: Is this really unreachable?
            _ => unreachable!(),
        }
    }

    fn ensure_lvalue_is_immutable(
        &mut self,
        lvalue: &Expr,
        line: usize,
        column: usize,
    ) -> Result<(), CompilerError> {
        let lvalue_symbol = self.lookup_lvalue(lvalue, line, column)?;

        match lvalue_symbol {
            Symbol::Var { name, mutable, .. } => {
                if !*mutable {
                    Ok(())
                } else {
                    Err(CompilerError::new(
                        &format!("Ang `{}` ay `maiba`", name),
                        ErrorKind::Error,
                        line,
                        column,
                    )
                    .add_help("Subukan mong tanggalin ang `maiba` sa deklarasyon nito"))
                }
            }
            // WARN: Is this really unreachable?
            _ => unreachable!(),
        }
    }

    fn enter_scope(&mut self) {
        self.parent_module.symbol_table.push(HashMap::new());
    }

    fn exit_scope(&mut self) {
        let _ = self.parent_module.symbol_table.pop();
    }

    pub fn has_error(&self) -> bool {
        self.has_error
    }
}
