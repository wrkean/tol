use std::fmt;

use crate::{lexer::token::Token, parser::ast::stmt::Stmt, toltype::TolType};

#[derive(Debug)]
pub enum Expr {
    IntLit(Token),
    FloatLit(Token),
    Identifier(Token),
    Binary {
        op: Token,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Block {
        statements: Vec<Stmt>,
        line: usize,
        column: usize,
    },
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::IntLit(tok) => write!(f, "{}", tok.lexeme()),
            Expr::FloatLit(tok) => write!(f, "{}", tok.lexeme()),
            Expr::Identifier(tok) => write!(f, "{}", tok.lexeme()),
            Expr::Binary {
                op, left, right, ..
            } => {
                write!(f, "({} {} {})", op.lexeme(), left, right)
            }
            _ => write!(f, ""),
        }
    }
}
