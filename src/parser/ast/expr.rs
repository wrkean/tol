use std::fmt;

use crate::{lexer::token::Token, toltype::TolType};

#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    IntLit {
        token: Token,
        id: usize,
    },
    FloatLit {
        token: Token,
        id: usize,
    },
    StringLit {
        token: Token,
        id: usize,
    },
    ByteStringLit {
        token: Token,
        id: usize,
    },
    Identifier {
        token: Token,
        id: usize,
    },
    Binary {
        op: Token,
        left: Box<Expr>,
        right: Box<Expr>,
        id: usize,
    },
    Assign {
        left: Box<Expr>,
        right: Box<Expr>,
        line: usize,
        column: usize,
        id: usize,
    },
    FnCall {
        callee: Token,
        args: Vec<Expr>,
        id: usize,
    },
    MagicFnCall {
        fncall: Box<Expr>,
        id: usize,
    },
    FieldAccess {
        left: Box<Expr>,
        member: Token,
        line: usize,
        column: usize,
        id: usize,
    },
    StaticFieldAccess {
        left: Token,
        field: Token,
        line: usize,
        column: usize,
        id: usize,
    },
    MethodCall {
        left: Box<Expr>,
        callee: Token,
        args: Vec<Expr>,
        line: usize,
        column: usize,
        id: usize,
    },
    StaticMethodCall {
        left: TolType,
        callee: Token,
        args: Vec<Expr>,
        line: usize,
        column: usize,
        id: usize,
    },
    Struct {
        name: TolType,
        fields: Vec<(Token, Expr)>,
        line: usize,
        column: usize,
        id: usize,
    },
    Array {
        elements: Vec<Expr>,
        line: usize,
        column: usize,
        id: usize,
    },
    RangeExclusive {
        start: Box<Expr>,
        end: Box<Expr>,
        line: usize,
        column: usize,
        id: usize,
    },
    RangeInclusive {
        start: Box<Expr>,
        end: Box<Expr>,
        line: usize,
        column: usize,
        id: usize,
    },
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::IntLit { token, .. } => write!(f, "{}", token.lexeme()),
            Expr::FloatLit { token, .. } => write!(f, "{}", token.lexeme()),
            Expr::Identifier { token, .. } => write!(f, "{}", token.lexeme()),
            Expr::Binary {
                op, left, right, ..
            } => {
                write!(f, "({} {} {})", op.lexeme(), left, right)
            }
            _ => write!(f, ""),
        }
    }
}

impl Expr {
    pub fn is_lvalue(&self) -> bool {
        matches!(
            self,
            Self::Identifier { .. } | Self::FieldAccess { .. } | Self::StaticFieldAccess { .. }
        )
    }
}
