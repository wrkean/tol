use std::collections::HashMap;

use crate::{symbol::Symbol, toltype::TolType};

pub struct TypeInfo {
    pub kind: TolType,
    pub fields: HashMap<String, TolType>,
    pub methods: HashMap<String, MethodInfo>,
}

pub struct MethodInfo {
    pub arg_types: Vec<TolType>,
    pub return_type: TolType,
    pub is_static: bool,
}
