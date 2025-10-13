use crate::{lexer::token::Token, parser::ast::expr::Expr, toltype::TolType};

#[derive(Debug, PartialEq, Clone)]
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
    Method {
        is_static: bool,
        met_identifier: Token,
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
    Bagay {
        bagay_identifier: Token,
        fields: Vec<(Token, TolType)>,
    },
    Itupad {
        itupad_for: TolType,
        itupad_block: Box<Stmt>,
        line: usize,
        column: usize,
    },
    ItupadBlock {
        methods: Vec<Stmt>,
        line: usize,
        column: usize,
    },
    Kung {
        branches: Vec<KungBranch>,
        line: usize,
        column: usize,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub struct KungBranch {
    pub condition: Option<Expr>,
    pub block: Expr,
}
