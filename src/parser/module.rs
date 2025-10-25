use std::{collections::HashMap, path::Path};

use crate::{
    parser::ast::stmt::Stmt,
    symbol::Symbol,
    toltype::{TolType, type_info::TypeInfo},
};

pub struct Module {
    pub source_path: String,
    pub module_name: String,
    pub ast: Vec<Stmt>,
    pub symbol_table: Vec<HashMap<String, Symbol>>,
    pub type_table: HashMap<String, TypeInfo>,
    pub inferred_types: HashMap<usize, TolType>,
    pub declared_array_types: Vec<String>,
}

impl Module {
    pub fn new(source_path: String, ast: Vec<Stmt>) -> Self {
        let module_name = Path::new(&source_path)
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();

        Self {
            source_path,
            ast,
            module_name,
            symbol_table: vec![HashMap::new()],
            type_table: HashMap::new(),
            inferred_types: HashMap::new(),
            declared_array_types: Vec::new(),
        }
    }
}
