use std::{collections::HashMap};
use crate::{ast::{FunctionDef}, compiler::{CompilationContext, Runable}};

pub struct CodeRepository {
    code: HashMap<String, Runable>
}

impl CodeRepository {
    pub fn new() -> CodeRepository {
        CodeRepository {
            code: HashMap::new(),
        }
    }
    
    pub fn add_placeholder(&mut self, function_def: &FunctionDef) -> Result<(), String> {
        let mut ctx = CompilationContext::new(self);
        if let Some(var) = function_def.parameter.clone() {
            ctx.assign_register_to_variable(var)?;
        }
        let compiled = ctx.compile(&function_def.body)?;
        self.code.insert(function_def.name.clone(), compiled);
        Ok(())
    }

    pub fn get_fn(&self, name: &str) -> Option<&Runable> {
        println!("Getting fn: {}, size: {}", name, self.code.len());
        self.code.get(name)
    }

    pub fn print_code(&self, name: &str) {
        match self.code.get(name) {
            Some(runable) => {
                runable.print();
            }
            None => {
                println!("No code entry found for fn {}.", name);
            }
        }
    }
}