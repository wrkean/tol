use std::collections::HashMap;

use crate::{symbol::Symbol, toltype::TolType};

#[derive(Debug, Clone)]
pub struct TypeInfo {
    pub kind: TolType,
    pub fields: HashMap<String, TolType>,
    pub methods: HashMap<String, Symbol>,
}

// #[derive(Debug, Clone)]
// pub struct MethodInfo {
//     pub param_types: Vec<TolType>,
//     pub return_type: TolType,
//     pub is_static: bool,
// }
