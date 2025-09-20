use crate::toltype::TolType;

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
}

impl Symbol {
    pub fn get_type(&self) -> &TolType {
        match self {
            Symbol::VarSymbol { tol_type, .. } => tol_type,
            Symbol::ParSymbol { return_type, .. } => return_type,
        }
    }
}
