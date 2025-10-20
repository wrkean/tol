use crate::toltype::TolType;

// FIXME: Varianrs have the same postfix warning. Not a problem for now
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Symbol {
    Var {
        name: String,
        mutable: bool,
        tol_type: TolType,
    },
    Paraan {
        name: String,
        param_types: Vec<TolType>,
        return_type: TolType,
    },
    Method {
        is_static: bool,
        name: String,
        param_types: Vec<TolType>,
        return_type: TolType,
    },
    Bagay {
        name: String,
    },
}

impl Symbol {
    pub fn get_type(&self) -> TolType {
        match self {
            Symbol::Var { tol_type, .. } => tol_type.to_owned(),
            Symbol::Paraan { return_type, .. } => return_type.to_owned(),
            Symbol::Bagay { name, .. } => TolType::Bagay(name.to_owned()),
            Symbol::Method { return_type, .. } => return_type.to_owned(),
        }
    }
}
