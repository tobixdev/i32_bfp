use crate::{ast, code_repository::CodeRepository, compiler::{CompilationContext, Runable}, runtime::Executor};

pub struct CompiledExecutor {
    code_repository: CodeRepository
}

impl CompiledExecutor {
    pub fn new() -> CompiledExecutor {
        CompiledExecutor {
            code_repository: CodeRepository::new()
        }
    }

    pub fn list_functions(&self) {
        self.code_repository.list_functions();
    }

    pub fn print_code(&self, name: &str) {
        self.code_repository.print_code(name);
    }
}

impl Executor for CompiledExecutor {
    fn handle_function_def(&mut self, func_def: ast::FunctionDef) -> Result<(), String> {
        Ok(self.code_repository.add_placeholder(func_def)?)
    }
    
    fn get_query_runable(&mut self, query: ast::Expr) -> Result<Runable, String> {
        let used_vars = query.used_variables();
        let mut ctx = CompilationContext::new(&mut self.code_repository);
    
        for used_var in &used_vars {
            ctx.assign_register_to_variable(used_var.to_string())?;
        }
    
        Ok(ctx.compile(&query)?)
    }

    fn delete(&mut self, name: &str) {
        self.code_repository.delete(name);
    }
}