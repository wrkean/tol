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
        id: usize,
    },
    Method {
        is_static: bool,
        met_identifier: Token,
        params: Vec<(Token, TolType)>,
        return_type: TolType,
        block: Expr,
        line: usize,
        column: usize,
        id: usize,
    },
    Ang {
        mutable: bool,
        ang_identifier: Token,
        ang_type: TolType,
        rhs: Expr,
        line: usize,
        column: usize,
        id: usize,
    },
    Ibalik {
        rhs: Expr,
        line: usize,
        column: usize,
        id: usize,
    },
    ExprS {
        expr: Expr,
        line: usize,
        column: usize,
        id: usize,
    },
    Bagay {
        bagay_identifier: Token,
        fields: Vec<(Token, TolType)>,
        id: usize,
    },
    Itupad {
        itupad_for: TolType,
        itupad_block: Box<Stmt>,
        line: usize,
        column: usize,
        id: usize,
    },
    ItupadBlock {
        methods: Vec<Stmt>,
        line: usize,
        column: usize,
        id: usize,
    },
    Kung {
        branches: Vec<KungBranch>,
        line: usize,
        column: usize,
        id: usize,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub struct KungBranch {
    pub condition: Option<Expr>,
    pub block: Expr,
}
