use std::collections::linked_list;

use crate::{
    lexer::token::Token,
    parser::ast::{expr::Expr, stmt::Stmt},
    toltype::TolType,
};

pub struct CodeGenerator<'a> {
    ast: &'a Stmt,
    output: String,
}

impl<'a> CodeGenerator<'a> {
    pub fn new(ast: &'a Stmt) -> Self {
        // Parents must exist first
        Self {
            ast,
            output: String::from(
                "#include<stdio.h>\n\
#include<stdlib.h>\n",
            ),
        }
    }

    pub fn generate(&mut self) -> &String {
        if let Stmt::Program(statements) = self.ast {
            for statement in statements {
                self.gen_statement(statement);
            }
        }

        // Call the main function defined by the user in C's main function
        self.output.push_str("int main(){__TOL_main__();return 0;}");

        println!("{}", self.output);

        &self.output
    }

    fn gen_statement(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Ang {
                mutable,
                ang_identifier,
                ang_type,
                rhs,
                ..
            } => {
                let modifier_c = if !mutable { "const " } else { "" };
                let type_c = ang_type.as_c();
                let id_c = ang_identifier.lexeme();
                let rhs_c = self.gen_expression(rhs);

                let ang_c = format!("{modifier_c}{type_c} {id_c} = {rhs_c};");

                self.output.push_str(&ang_c);
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

                let par_c = format!("{type_c} {id_c}{params_c}");
                self.output.push_str(&par_c);
                self.gen_expression(block);
            }
            Stmt::Ibalik { rhs, .. } => {
                let ibalik_c = format!("return {};", self.gen_expression(rhs));
                self.output.push_str(&ibalik_c);
            }
            Stmt::ExprS { expr, .. } => {
                let expr_c = self.gen_expression(expr);
                self.output.push_str(&expr_c);
                self.output.push(';');
            }
            Stmt::Bagay {
                bagay_identifier,
                fields,
            } => {
                let bagay_id_c = bagay_identifier.lexeme();
                let mut fields_c = String::new();
                for field in fields {
                    fields_c.push_str(&format!("{} {};", field.1.as_c(), field.0.lexeme()));
                }
                self.output.push_str(&format!(
                    "typedef struct {bagay_id_c}{{{fields_c}}}{bagay_id_c};"
                ));
            }
            Stmt::Itupad {
                itupad_for,
                itupad_block,
                ..
            } => {
                if let Stmt::ItupadBlock { methods, .. } = &**itupad_block {
                    for method in methods {
                        self.gen_method(method, itupad_for);
                    }
                }
            }
            Stmt::Program(_) => {}
            _ => {}
        }
    }

    fn gen_method(&mut self, method: &Stmt, itupad_for: &TolType) {
        if let Stmt::Method {
            is_static,
            met_identifier,
            params,
            return_type,
            block,
            line,
            column,
        } = method
        {
            let type_c = return_type.as_c();
            let id_c = met_identifier.lexeme();
            let params_c = self.gen_params(params, Some(itupad_for));

            self.output.push_str(&format!("{type_c} {id_c}{params_c}"));
            self.gen_expression(block);
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

    fn gen_expression(&mut self, expr: &Expr) -> String {
        match expr {
            Expr::IntLit(tok) | Expr::FloatLit(tok) | Expr::Identifier(tok) => {
                tok.lexeme().to_string()
            }
            Expr::StringLit(tok) => {
                format!("\"{}\"", tok.lexeme())
            }
            Expr::Binary { op, left, right } => {
                format!(
                    "({} {} {})",
                    self.gen_expression(left),
                    op.lexeme(),
                    self.gen_expression(right)
                )
            }
            Expr::Block { statements, .. } => {
                self.output.push('{');
                for statement in statements {
                    self.gen_statement(statement);
                }
                self.output.push('}');

                String::from("")
            }
            Expr::FnCall { callee, args } => {
                let mut args_str_c = String::from("(");
                for arg in args {
                    args_str_c.push_str(&self.gen_expression(arg));
                    if arg != args.last().unwrap() {
                        args_str_c.push_str(", ");
                    }
                }
                args_str_c.push(')');

                format!("{}{}", callee.lexeme(), args_str_c)
            }
            Expr::MagicFnCall { fncall } => {
                if let Expr::FnCall { callee, args } = fncall.as_ref() {
                    match callee.lexeme() {
                        "print" => {
                            let str_arg = self.gen_expression(&args[0]);
                            format!("fputs({str_arg}, stdout)")
                        }
                        "println" => {
                            let str_arg = self.gen_expression(&args[0]);
                            format!("puts({str_arg})")
                        }
                        "exit" => {
                            let int_arg = self.gen_expression(&args[0]);
                            format!("exit({int_arg})")
                        }
                        _ => "".to_owned(),
                    }
                } else {
                    panic!("MagicFnCall did not contain a function call!")
                }
            }
            Expr::FieldAccess { left, member, .. } => {
                let left_expr_c = self.gen_expression(left);
                let right_member_c = member.lexeme();

                format!("({left_expr_c}.{right_member_c})")
            }
            Expr::MethodCall {
                left, callee, args, ..
            } => {
                let mut args_str_c = String::from("(");
                args_str_c.push_str(&self.gen_expression(left));
                for arg in args {
                    args_str_c.push_str(&self.gen_expression(arg));
                    if arg != args.last().unwrap() {
                        args_str_c.push_str(", ");
                    }
                }
                args_str_c.push(')');

                format!("{}{}", callee.lexeme(), args_str_c)
            }
            Expr::StaticMethodCall { callee, args, .. } => {
                let mut args_str_c = String::from("(");
                for arg in args {
                    args_str_c.push_str(&self.gen_expression(arg));
                    if arg != args.last().unwrap() {
                        args_str_c.push_str(", ");
                    }
                }
                args_str_c.push(')');

                format!("{}{}", callee.lexeme(), args_str_c)
            }
            Expr::Struct { name, fields } => {
                let struct_name_c = name.lexeme();
                let mut struct_block_c = String::from("{");
                for (i, (field_name, field_expr)) in fields.iter().enumerate() {
                    struct_block_c.push_str(&format!(
                        ".{}={}",
                        field_name.lexeme(),
                        self.gen_expression(field_expr)
                    ));

                    let is_last = i == fields.len() - 1;
                    if !is_last {
                        struct_block_c.push(',');
                    }
                }
                struct_block_c.push('}');

                format!("(struct {}){}", struct_name_c, struct_block_c)
            }
            _ => String::from("Wala"),
        }
    }
}
