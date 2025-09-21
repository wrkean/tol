use std::{
    fs::{self, File},
    io::{self, Write},
    path::PathBuf,
};

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
            output: String::from("#include<stdio.h>\n"),
        }
    }

    pub fn generate(&mut self) -> &String {
        if let Stmt::Program(statements) = self.ast {
            for statement in statements {
                self.gen_statement(statement);
            }
        }

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
                let params_c = self.gen_params(params);

                let par_c = format!("{type_c} {id_c}{params_c} ");
                self.output.push_str(&par_c);
                self.gen_expression(block);
            }
            Stmt::Ibalik { rhs, .. } => {
                let ibalik_c = format!("return {};", self.gen_expression(rhs));
                self.output.push_str(&ibalik_c);
            }
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
        }
    }
}
