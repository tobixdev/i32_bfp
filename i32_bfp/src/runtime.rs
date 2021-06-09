use crate::parser::parse;
use crate::code_repository::CodeRepository;
use crate::compiler::CompilationContext;
use crate::ast;

#[derive(Debug)]
enum ExeuctionMode {
    Proof,
    Fast
}

impl ExeuctionMode {
    fn from(value: &str) -> ExeuctionMode {
        match value {
            "proof" => ExeuctionMode::Proof,
            "fast" => ExeuctionMode::Fast,
            _ => panic!("Invalid exeuction mode!")
        }
    }
}

pub struct Runtime {
    mode: ExeuctionMode,
    code_repository: CodeRepository
}

impl Runtime {
    
    pub fn new() -> Runtime {
        Runtime {
            mode: ExeuctionMode::Proof,
            code_repository: CodeRepository::new()
        }
    }

    pub fn handle_line(&mut self, line: &str) {
        let result = parse(line)
            .and_then(|ast| self.handle_ast(ast));
        if let Err(error) = result {
            println!("ERROR>\n{}", error);
        }
    }
    
    fn handle_ast(&mut self, ast: ast::Action) -> Result<(), String> {
        match ast {
            ast::Action::FunctionDef(func_def) => self.code_repository.add_placeholder(func_def)?,
            ast::Action::Query(query) => self.execute_query(query)?,
            ast::Action::Command(ast::Command::ShowCode(name)) => self.code_repository.print_code(&name),
            ast::Action::Command(ast::Command::ListFunctions()) => self.code_repository.list_functions(),
            ast::Action::Command(ast::Command::DeleteFunction(name)) => self.code_repository.delete(&name),
            ast::Action::Command(ast::Command::SwitchMode(mode)) => {
                self.mode = ExeuctionMode::from(&mode);
                println!("Switched mode to {:?}", self.mode);
            }
        }
        Ok(())
    }
    
    fn execute_query(&mut self, query: ast::Expr) -> Result<(), String> {
        let used_vars = query.used_variables();
        let mut ctx = CompilationContext::new(&mut self.code_repository);
    
        for used_var in &used_vars {
            ctx.assign_register_to_variable(used_var.to_string())?;
        }
    
        let runable = ctx.compile(&query)?;
        println!("The following free variables were found: {:?}", used_vars);
        let (first_var_range, mut to_check) = self.get_first_var_range(&used_vars);
        println!("{} loops remaining...", to_check);
        for i in first_var_range {
            if to_check % 100_000_000 == 0 {
                println!("{} loops remaining...", to_check)
            }
            let result = runable.call(i);
            if result == 0 {
                println!("Formula does not hold for {}!", i);
                return Ok(());
            }
            to_check-=1;
        }
        println!("Formula does hold.");
        Ok(())
    }

    fn get_first_var_range(&self, used_vars: &Vec<String>) -> (Box<dyn Iterator<Item = i32>>, usize) {
        if used_vars.len() == 0 {
            return (Box::new(0..=0), 1);
        }

        match self.mode {
            ExeuctionMode::Proof => (Box::new(i32::MIN..=i32::MAX), (i32::MAX as usize) + 1 + -(i32::MIN as i64) as usize),
            ExeuctionMode::Fast => (Box::new(vec![i32::MIN, -1, 0, 1, i32::MAX].into_iter()), 5)
        }
    }
}
