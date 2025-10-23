use std::fmt;

use crate::lexer::token::Token;

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
        callee: Box<Expr>,
        args: Vec<Expr>,
        line: usize,
        column: usize,
        id: usize,
    },
    MagicFnCall {
        name: Token,
        args: Vec<Expr>,
        id: usize,
    },
    MemberAccess {
        left: Box<Expr>,
        member: Token,
        line: usize,
        column: usize,
        id: usize,
    },
    ScopeResolution {
        left: Box<Expr>,
        field: Token,
        line: usize,
        column: usize,
        id: usize,
    },
    Struct {
        callee: Box<Expr>,
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
    AddressOf {
        of: Box<Expr>,
        line: usize,
        column: usize,
    },
    Deref {
        right: Box<Expr>,
        line: usize,
        column: usize,
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
            Self::Identifier { .. }
                | Self::MemberAccess { .. }
                | Self::ScopeResolution { .. }
                | Self::Deref { .. },
        )
    }
}
