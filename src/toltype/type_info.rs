use std::collections::HashMap;

use crate::toltype::TolType;

pub struct TypeInfo {
    pub kind: TolType,
    pub fields: HashMap<String, TolType>,
}
