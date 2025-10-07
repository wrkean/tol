use std::fmt;

use crate::{lexer::token::Token, parser::ast::stmt::Stmt};

#[derive(Debug, PartialEq)]
pub enum Expr {
    IntLit(Token),
    FloatLit(Token),
    StringLit(Token),
    Identifier(Token),
    Binary {
        op: Token,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    FnCall {
        callee: Token,
        args: Vec<Expr>,
    },
    MagicFnCall {
        fncall: Box<Expr>,
    },
    Block {
        statements: Vec<Stmt>,
        line: usize,
        column: usize,
    },
    FieldAccess {
        left: Box<Expr>,
        member: Token,
        line: usize,
        column: usize,
    },
    StaticFieldAccess {
        left: Box<Expr>,
        field: Token,
        line: usize,
        column: usize,
    }, // MethodCall {
       //     left: Box<Expr>,
       //     method: Token,
       //     args: Vec<Expr>,
       //     line: usize,
       //     column: usize,
       // },
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
