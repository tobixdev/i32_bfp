use std::{collections::HashMap};
use crate::{ast::{FunctionDef}, compiler::CompilationContext};

pub struct CodeRepository {
    code: HashMap<String, extern "win64" fn(i32) -> i32>
}

impl CodeRepository {
    pub fn new() -> CodeRepository {
        CodeRepository {
            code: HashMap::new(),
        }
    }
    
    pub fn add_placeholder(&mut self, function_def: &FunctionDef) {
        let compiled = CompilationContext::new().compile(function_def.body.as_ref());
        self.code.insert(function_def.name.clone(), compiled);
    }
}