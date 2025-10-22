use std::collections::HashMap;

use crate::{symbol::Symbol, toltype::TolType};

#[derive(Debug, Clone)]
pub struct TypeInfo {
    pub kind: TolType,
    pub members: HashMap<String, Symbol>,
    pub static_members: HashMap<String, Symbol>,
}

impl TypeInfo {
    pub fn new(kind: TolType) -> Self {
        Self {
            kind,
            members: HashMap::new(),
            static_members: HashMap::new(),
        }
    }
}

// #[derive(Debug, Clone)]
// pub struct MethodInfo {
//     pub param_types: Vec<TolType>,
//     pub return_type: TolType,
//     pub is_static: bool,
// }
