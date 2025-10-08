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
                let params_c = self.gen_params(params);

                let par_c = format!("{type_c} {id_c}{params_c} ");
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
            Stmt::Method {
                met_identifier,
                params,
                return_type,
                block,
                ..
            } => {
                let type_c = return_type.as_c();
                let id_c = met_identifier.lexeme();
                let params_c = self.gen_params(params);
                let met_c = format!("{type_c} {id_c}{params_c}");
                self.output.push_str(&met_c);
                self.gen_expression(block);
            }
            Stmt::Program(_) => {}
            _ => {}
        }
    }

    fn gen_params(&self, params: &[(Token, TolType)]) -> String {
        let mut c_params = String::from("(");
        for param in params {
            c_params += &format!("{} {}", param.1.as_c(), param.0.lexeme());
            if param != params.last().unwrap() {
                c_params += ", ";
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
            _ => String::from("Wala"),
        }
    }
}
