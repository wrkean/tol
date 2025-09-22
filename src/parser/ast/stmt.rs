use crate::{lexer::token::Token, parser::ast::expr::Expr, toltype::TolType};

#[derive(Debug, PartialEq)]
pub enum Stmt {
    Program(Vec<Stmt>),
    Par {
        par_identifier: Token,
        params: Vec<(Token, TolType)>,
        return_type: TolType,
        block: Expr,
        line: usize,
        column: usize,
    },
    Ang {
        mutable: bool,
        ang_identifier: Token,
        ang_type: TolType,
        rhs: Expr,
        line: usize,
        column: usize,
    },
    Ibalik {
        rhs: Expr,
        line: usize,
        column: usize,
    },
    ExprS {
        expr: Expr,
        line: usize,
        column: usize,
    },
}
