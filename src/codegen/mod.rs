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
    project_root: PathBuf,
    output_file: File,
    output_path: PathBuf,
}

impl<'a> CodeGenerator<'a> {
    pub fn new(ast: &'a Stmt, project_root_str: &'a str, filename: &str) -> io::Result<Self> {
        let project_root = fs::canonicalize(project_root_str)?;
        let output_path = project_root.join("build/output/c").join(filename);

        // Parents must exist first
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut output_file = File::create(&output_path)?;
        let _ = output_file.write("#include<stdint.h>\n".as_bytes());

        Ok(Self {
            ast,
            project_root,
            output_file,
            output_path,
        })
    }

    pub fn generate(&mut self) -> &PathBuf {
        if let Stmt::Program(statements) = self.ast {
            for statement in statements {
                self.gen_statement(statement);
            }
        }

        &self.output_path
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

                let _ = self.output_file.write(ang_c.as_bytes());
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
                let return_type_c = return_type.as_c();

                let par_c = format!("{type_c} {id_c}{params_c} ");

                let _ = self.output_file.write(par_c.as_bytes());
                let _ = self.gen_expression(block);
            }
            Stmt::Ibalik { rhs, line, column } => {
                let ibalik_c = format!("return {};", self.gen_expression(rhs));

                let _ = self.output_file.write(ibalik_c.as_bytes());
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
                let _ = self.output_file.write("{".as_bytes());

                for statement in statements {
                    self.gen_statement(statement);
                }

                let _ = self.output_file.write("}".as_bytes());
                String::from("")
            }
        }
    }
}
