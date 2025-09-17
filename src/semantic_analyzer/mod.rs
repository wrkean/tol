use crate::{
    lexer::token::Token,
    parser::ast::{expr::Expr, stmt::Stmt},
    tol::toltype::TolType,
};

pub struct SemanticAnalyzer {
    ast: Stmt,
}

impl SemanticAnalyzer {
    pub fn new(ast: Stmt) -> Self {
        Self { ast }
    }

    pub fn analyze(&self) {
        if let Stmt::Program(statements) = &self.ast {
            for stmt in statements {
                self.analyze_stmt(stmt);
            }
        }
    }

    fn analyze_stmt(&self, stmt: &Stmt) {
        match stmt {
            Stmt::Ang {
                mutable,
                ang_identifier,
                ang_type,
                rhs,
                line,
                column,
            } => self.analyze_ang(mutable, ang_identifier, ang_type, rhs, line, column),
            Stmt::Par {
                par_identifier,
                params,
                return_type,
                block,
                line,
                column,
            } => self.analyze_par(par_identifier, params, return_type, block, line, column),
            _ => {}
        }
    }

    fn analyze_ang(
        &self,
        mutable: &bool,
        ang_identifier: &Token,
        ang_type: &TolType,
        rhs: &Expr,
        line: &usize,
        column: &usize,
    ) {
    }

    fn analyze_par(
        &self,
        ang_identifier: &Token,
        params: &[(Token, TolType)],
        return_type: &TolType,
        block: &Expr,
        line: &usize,
        column: &usize,
    ) {
    }

    fn analyze_expression(&self, expr: &Expr) -> TolType {
        todo!()
    }
}
