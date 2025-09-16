use std::fmt;

use crate::{lexer::token::Token, parser::ast::stmt::Stmt, tol::toltype::TolType};

#[derive(Debug)]
pub enum Expr {
    IntLit(Token, TolType),
    FloatLit(Token, TolType),
    Identifier(Token, TolType),
    Binary {
        op: Token,
        left: Box<Expr>,
        right: Box<Expr>,
        expr_type: TolType,
    },
    Block {
        statements: Vec<Stmt>,
        line: usize,
        column: usize,
        expr_type: TolType,
    },
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::IntLit(tok, _) => write!(f, "{}", tok.lexeme()),
            Expr::FloatLit(tok, _) => write!(f, "{}", tok.lexeme()),
            Expr::Identifier(tok, _) => write!(f, "{}", tok.lexeme()),
            Expr::Binary {
                op, left, right, ..
            } => {
                write!(f, "({} {} {})", op.lexeme(), left, right)
            }
            _ => write!(f, ""),
        }
    }
}
