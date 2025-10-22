use std::collections::HashMap;

use crate::{
    lexer::token::Token,
    parser::ast::{expr::Expr, stmt::Stmt},
    toltype::TolType,
};

pub struct CodeGenerator<'a> {
    ast: &'a Stmt,
    output: String,
    inferred_types: &'a HashMap<usize, TolType>,
}

impl<'a> CodeGenerator<'a> {
    pub fn new(ast: &'a Stmt, inferred_types: &'a HashMap<usize, TolType>) -> Self {
        // Parents must exist first
        Self {
            ast,
            output: String::from(
                "#include<stdio.h>\n\
#include<stdlib.h>\n",
            ),
            inferred_types,
        }
    }

    pub fn generate(&mut self) -> &String {
        if let Stmt::Program(statements) = self.ast {
            self.output.push_str(&self.gen_statements(statements));
        }

        // Call the main function defined by the user in C's main function
        self.output.push_str("int main(){__TOL_main__();return 0;}");

        println!("{}", self.output);

        &self.output
    }

    fn gen_statements(&self, statements: &[Stmt]) -> String {
        let mut out = String::new();
        for stmt in statements {
            out.push_str(&self.gen_statement(stmt));
        }

        out
    }

    fn gen_statement(&self, stmt: &Stmt) -> String {
        match stmt {
            Stmt::Ang {
                mutable,
                ang_identifier,
                ang_type,
                rhs,
                id,
                ..
            } => {
                let modifier_c = if !mutable { "const " } else { "" };
                let ang_type = match ang_type {
                    TolType::Unknown => self.get_inferred_type(*id),
                    _ => ang_type,
                };
                let type_c = ang_type.as_c();
                let id_c = format!("{}{}", ang_identifier.lexeme(), ang_type.array_suffix());
                let rhs_c = self.gen_expression(rhs);

                format!("{modifier_c}{type_c} {id_c} = {rhs_c};")
            }
            Stmt::Par {
                par_identifier,
                params,
                return_type,
                block,
                ..
            } => {
                let type_c = return_type.as_c();
                let id_c = par_identifier.lexeme();
                let id_c = match id_c {
                    "una" => "__TOL_main__",
                    _ => id_c,
                };
                let params_c = self.gen_params(params, None);
                let block_c = self.gen_block(block);

                format!("{type_c} {id_c}{params_c}{block_c}")
            }
            Stmt::Ibalik { rhs, .. } => {
                format!("return {};", self.gen_expression(rhs))
            }
            Stmt::ExprS { expr, .. } => {
                format!("{};", self.gen_expression(expr))
            }
            Stmt::Bagay {
                bagay_identifier,
                fields,
                ..
            } => {
                let bagay_id_c = bagay_identifier.lexeme();
                let mut fields_c = String::new();
                for field in fields {
                    fields_c.push_str(&format!("{} {};", field.1.as_c(), field.0.lexeme()));
                }
                format!("typedef struct {bagay_id_c}{{{fields_c}}}{bagay_id_c};")
            }
            Stmt::Itupad {
                itupad_for,
                itupad_block,
                ..
            } => {
                if let Stmt::ItupadBlock { methods, .. } = &**itupad_block {
                    let mut out = String::new();
                    for method in methods {
                        out.push_str(&self.gen_method(method, itupad_for));
                    }

                    out
                } else {
                    unreachable!("Itupadblock mismatch");
                }
            }
            Stmt::Kung { branches, .. } => {
                let mut if_c = String::new();
                for (i, branch) in branches.iter().enumerate() {
                    if i == 0 {
                        if_c.push_str(&format!(
                            "if ({}){}",
                            self.gen_expression(branch.condition.as_ref().unwrap()),
                            self.gen_block(&branch.block)
                        ));
                    } else if let Some(expr) = &branch.condition {
                        if_c.push_str(&format!(
                            "else if ({}){}",
                            self.gen_expression(expr),
                            self.gen_block(&branch.block)
                        ));
                    } else if branch.condition.is_none() {
                        if_c.push_str(&format!("else {}", self.gen_block(&branch.block)));
                    }
                }

                if_c
            }
            Stmt::Sa {
                iterator,
                bind,
                block,
                id,
                ..
            } => match iterator {
                Expr::RangeExclusive { start, end, .. } => {
                    let bind_type = self.get_inferred_type(*id).as_c();
                    let bind_id_c = bind.lexeme();
                    let start_c = self.gen_expression(start);
                    let end_c = self.gen_expression(end);
                    let block_c = self.gen_block(block);

                    format!(
                        "for ({bind_type} {bind_id_c} = {start_c}; {bind_id_c} < {end_c}; {bind_id_c}++) {block_c}"
                    )
                }
                Expr::RangeInclusive { start, end, .. } => {
                    let bind_type = self.get_inferred_type(*id).as_c();
                    let bind_id_c = bind.lexeme();
                    let start_c = self.gen_expression(start);
                    let end_c = self.gen_expression(end);
                    let block_c = self.gen_block(block);

                    format!(
                        "for ({bind_type} {bind_id_c} = {start_c}; {bind_id_c} <= {end_c}; {bind_id_c}++) {block_c}"
                    )
                }
                _ => {
                    panic!(
                        "Hindi muna pwede ang ibang expresyon bukod sa `..` sa `sa`, ito ay gagawin pa. :)"
                    );
                }
            },
            Stmt::Program(statements) => self.gen_statements(statements),
            _ => "".to_string(),
        }
    }

    fn gen_method(&self, method: &Stmt, itupad_for: &TolType) -> String {
        if let Stmt::Method {
            met_identifier,
            params,
            return_type,
            block,
            ..
        } = method
        {
            let type_c = return_type.as_c();
            let id_c = met_identifier.lexeme();
            let params_c = self.gen_params(params, Some(itupad_for));
            let block_c = self.gen_block(block);

            format!("{type_c} {id_c}{params_c}{block_c}")
        } else {
            unreachable!("Stmt is not a method");
        }
    }

    fn gen_params(&self, params: &[(Token, TolType)], itupad_for: Option<&TolType>) -> String {
        let mut c_params = String::from("(");
        for (param_name, param_type) in params {
            if let TolType::AkoType = param_type {
                if let Some(t) = itupad_for {
                    c_params.push_str(&format!("{} {}", t.as_c(), param_name.lexeme()));
                }
            } else {
                c_params.push_str(&format!("{} {}", param_type.as_c(), param_name.lexeme()));
            }

            if param_name.lexeme() != params.last().unwrap().0.lexeme() {
                c_params.push(',');
            }
        }
        c_params += ")";

        c_params
    }

    fn gen_expression(&self, expr: &Expr) -> String {
        match expr {
            Expr::IntLit { token, .. }
            | Expr::FloatLit { token, .. }
            | Expr::Identifier { token, .. } => token.lexeme().to_string(),
            Expr::StringLit { token, .. } | Expr::ByteStringLit { token, .. } => {
                format!("\"{}\"", token.lexeme())
            }
            Expr::Binary {
                op, left, right, ..
            } => {
                format!(
                    "({} {} {})",
                    self.gen_expression(left),
                    op.lexeme(),
                    self.gen_expression(right)
                )
            }
            Expr::Assign { left, right, .. } => {
                format!(
                    "{} = {}",
                    self.gen_expression(left),
                    self.gen_expression(right)
                )
            }
            Expr::FnCall { callee, args, .. } => match callee.as_ref() {
                Expr::Identifier { token, .. } => {
                    format!("{}({})", token.lexeme(), self.gen_args(args))
                }
                Expr::MemberAccess { left, member, .. } => {
                    let mut out = format!("{}({}", member.lexeme(), self.gen_expression(left));
                    if !args.is_empty() {
                        out.push_str(&format!(", {})", self.gen_args(args)));
                    } else {
                        out.push(')');
                    }
                    out
                }
                Expr::ScopeResolution { field, .. } => {
                    format!("{}({})", field.lexeme(), self.gen_args(args))
                }
                _ => unreachable!(),
            },
            Expr::MemberAccess { left, member, .. } => {
                format!("{}.{}", self.gen_expression(left), member.lexeme())
            }
            Expr::ScopeResolution { field, .. } => field.lexeme().to_string(),
            Expr::MagicFnCall { name, args, .. } => {
                let args_c = self.gen_args(args);
                match name.lexeme() {
                    "println" => format!("fputs({}, stdout)", args_c),
                    "print" => format!("puts({})", args_c),
                    "alis" => format!("exit({})", args_c),
                    _ => unreachable!(),
                }
            }
            Expr::Struct { callee, fields, .. } => {
                let callee_c = self.gen_expression(callee);
                let mut fields_c = String::new();
                for (i, (tok, ex)) in fields.iter().enumerate() {
                    let field_c = format!(".{} = {}", tok.lexeme(), self.gen_expression(ex));
                    fields_c.push_str(&field_c);
                    if i != fields.len() - 1 {
                        fields_c.push(',')
                    }
                }

                format!("(struct {}){{ {} }}", callee_c, fields_c)
            }
            Expr::Array { elements, .. } => {
                let mut array_c = String::from("{");

                for (i, element) in elements.iter().enumerate() {
                    array_c.push_str(&self.gen_expression(element));
                    if i != elements.len() - 1 {
                        array_c.push_str(", ");
                    }
                }
                array_c.push('}');

                array_c
            }
            Expr::RangeExclusive { .. } => unimplemented!(),
            Expr::RangeInclusive { .. } => unimplemented!(),
        }
    }

    fn gen_args(&self, args: &[Expr]) -> String {
        let mut out = String::new();
        for (i, arg) in args.iter().enumerate() {
            out.push_str(&self.gen_expression(arg));
            if i != args.len() - 1 {
                out.push(',');
            }
        }

        out
    }

    fn gen_block(&self, block: &Stmt) -> String {
        if let Stmt::Block { statements, .. } = block {
            let mut out = String::from("{");
            for statement in statements {
                out.push_str(&self.gen_statement(statement));
            }

            out.push('}');

            out
        } else {
            unreachable!("block is not Expr::Block")
        }
    }

    fn get_inferred_type(&self, id: usize) -> &TolType {
        // println!("Getting id: {}", id);
        self.inferred_types.get(&id).unwrap()
    }
}
