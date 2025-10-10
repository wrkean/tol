use crate::toltype::TolType;

// FIXME: Varianrs have the same postfix warning. Not a problem for now
#[derive(Debug, Clone)]
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
    MetSymbol {
        is_static: bool,
        name: String,
        param_types: Vec<TolType>,
        return_type: TolType,
    },
    BagaySymbol {
        name: String,
    },
}

impl Symbol {
    pub fn get_type(&self) -> TolType {
        match self {
            Symbol::VarSymbol { tol_type, .. } => tol_type.to_owned(),
            Symbol::ParSymbol { return_type, .. } => return_type.to_owned(),
            Symbol::BagaySymbol { name, .. } => TolType::Bagay(name.to_owned()),
            Symbol::MetSymbol { return_type, .. } => return_type.to_owned(),
        }
    }
}
