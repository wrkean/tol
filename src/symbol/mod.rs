use crate::{lexer::token::Token, toltype::TolType};

pub enum Symbol {
    VarSymbol {
        name: String,
        tol_type: TolType,
    },
    ParSymbol {
        name: String,
        param_types: Vec<TolType>,
        return_type: TolType,
    },
    BagaySymbol {
        name: String,
        fields: Vec<(Token, TolType)>,
    },
}

impl Symbol {
    pub fn get_type(&self) -> TolType {
        match self {
            Symbol::VarSymbol { tol_type, .. } => tol_type.to_owned(),
            Symbol::ParSymbol { return_type, .. } => return_type.to_owned(),
            Symbol::BagaySymbol { name, .. } => TolType::Bagay(name.to_owned()),
        }
    }
}
