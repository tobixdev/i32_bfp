use std::time;

use crate::ast::{Expr};
use crate::compiled_executor::CompiledExecutor;
use crate::interpreted_executor::InterpretedExecutor;
use crate::parser::parse;
use crate::ast;

#[derive(Debug)]
enum ExecutorType {
    Compiled,
    Interpreted,
}

impl ExecutorType {
    fn from(value: &str) -> ExecutorType {
        match value {
            "compiled" => ExecutorType::Compiled,
            "interpreted" => ExecutorType::Interpreted,
            _ => panic!("Invalid exeuction mode!")
        }
    }
}

#[derive(Debug)]
enum ExeuctionMode {
    Proof,
    Fast,
    Benchmark
}

impl ExeuctionMode {
    fn from(value: &str) -> ExeuctionMode {
        match value {
            "proof" => ExeuctionMode::Proof,
            "fast" => ExeuctionMode::Fast,
            "benchmark" => ExeuctionMode::Benchmark,
            _ => panic!("Invalid exeuction mode!")
        }
    }

    fn should_print_info(&self) -> bool {
        match self {
            ExeuctionMode::Benchmark => false,
            _ => true
        }
    }
}

pub trait Executor {
    fn handle_function_def(&mut self, func_def: ast::FunctionDef) -> Result<(), String>;
    fn get_query_runable<'a>(&'a mut self, query: ast::Expr) -> Result<Box<dyn 'a + Fn(i32) -> i32>, String>;
    fn delete(&mut self, name: &str);
}

pub struct Runtime {
    mode: ExeuctionMode,
    used_executor: ExecutorType,
    compiled: CompiledExecutor,
    interpreted: InterpretedExecutor
}

impl Runtime {
    
    pub fn new() -> Runtime {
        Runtime {
            mode: ExeuctionMode::Proof,
            used_executor: ExecutorType::Compiled,
            compiled: CompiledExecutor::new(),
            interpreted: InterpretedExecutor::new()
        }
    }

    pub fn handle_line(&mut self, line: &str) {
        let result = parse(line)
            .and_then(|ast| self.handle_ast(ast));
        if let Err(error) = result {
            println!("ERROR>\n{}", error);
        }
    }

    fn handle_str(&mut self, str: &str) -> Result<(), String> {
        self.handle_ast(parse(str)?)
    }
    
    fn handle_ast(&mut self, ast: ast::Action) -> Result<(), String> {
        match ast {
            ast::Action::FunctionDef(func_def) => {
                self.compiled.handle_function_def(func_def.clone())?;
                self.interpreted.handle_function_def(func_def)?;
            },
            ast::Action::Query(query) => self.execute_query(query)?,
            ast::Action::Command(ast::Command::ShowCode(name)) => self.compiled.print_code(&name),
            ast::Action::Command(ast::Command::ListFunctions()) => self.compiled.list_functions(),
            ast::Action::Command(ast::Command::DeleteFunction(name)) => { 
                self.compiled.delete(&name);
             },
            ast::Action::Command(ast::Command::SwitchMode(mode)) => {
                self.mode = ExeuctionMode::from(&mode);
                println!("Switched mode to {:?}", self.mode);
            },
            ast::Action::Command(ast::Command::SwitchExecutor(executor)) => {
                self.used_executor = ExecutorType::from(&executor);
                println!("Switched executor to {:?}", self.used_executor);
            },
            ast::Action::Command(ast::Command::Test(expr)) => self.test_expr(&expr)?,
            ast::Action::Command(ast::Command::Benchmark) => self.benchmark()?
        }
        Ok(())
    }
    
    fn execute_query(&mut self, query: ast::Expr) -> Result<(), String> {
        let used_vars = query.used_variables();
        let (first_var_range, mut to_check) = self.get_first_var_range(&used_vars);

        let runable = match self.used_executor {
            ExecutorType::Compiled => self.compiled.get_query_runable(query)?,
            ExecutorType::Interpreted => self.interpreted.get_query_runable(query)?
        };

        println!("The following free variables were found: {:?}", used_vars);
        println!("Using {:?} executor...", self.used_executor);
        println!("{} loops remaining...", to_check);
        for i in first_var_range {
            if to_check % 100_000_000 == 0 && self.mode.should_print_info()  {
                println!("{} loops remaining...", to_check)
            }
            let result = runable(i);
            if result == 0 {
                println!("Formula does not hold for {}!", i);
                return Ok(());
            }
            to_check-=1;
        }
        println!("Formula does hold.");
        Ok(())
    }

    fn test_expr(&mut self, expr: &Expr) -> Result<(), String> {
        let compiler = self.compiled.get_query_runable(expr.clone())?;
        let interpreted = self.interpreted.get_query_runable(expr.clone())?;

        for i in -1000..1000 {
            let result_compiler = compiler(i);
            let result_interpreted = interpreted(i);
            if result_compiler != result_interpreted {
                println!("Difference between compiled and interpreted exeuction for input {}. Compiled: {}, Interpredted: {}.", i, result_compiler, result_interpreted);
                return Ok(())
            }
        }
        println!("Test OK");
        Ok(())
    }

    fn benchmark(&mut self) -> Result<(), String> {
        self.handle_str(".mode benchmark")?;
        self.execute_benchmark("Simple", "x <> x + 1")?;
        self.execute_benchmark("Complex", "(x + 1) % 2 <> x % 2")?;
        self.handle_str("f(x) := x * 2")?;
        self.execute_benchmark("Function Call", "x * 2 = f(x)")?;
        Ok(())
    }

    fn execute_benchmark(&mut self, name: &str, expression: &str) -> Result<(), String> {
        let expr = parse(expression)?;
        if let ast::Action::Query(query) = expr {
            self.handle_str(".executor compiled")?;
            let start = time::SystemTime::now();
            self.execute_query(query.clone())?;
            let expired = start.elapsed().unwrap().as_millis();
            println!("Benchmark '{}' took {} ms for 10.000.000 iterations in compiled mode.", name, expired);

            self.handle_str(".executor interpreted")?;
            let start = time::SystemTime::now();
            self.execute_query(query)?;
            let expired = start.elapsed().unwrap().as_millis();
            println!("Benchmark '{}' took {} ms for 10.000.000 iterations in interpreted mode.", name, expired);
        } else {
            println!("expression is not a query");
        }
        Ok(())
    }

    fn get_first_var_range(&self, used_vars: &Vec<String>) -> (Box<dyn Iterator<Item = i32>>, usize) {
        if used_vars.len() == 0 {
            return (Box::new(0..=0), 1);
        }

        match self.mode {
            ExeuctionMode::Proof => (Box::new(i32::MIN..=i32::MAX), (i32::MAX as usize) + 1 + -(i32::MIN as i64) as usize),
            ExeuctionMode::Fast => (Box::new(vec![i32::MIN, -1, 0, 1, i32::MAX].into_iter()), 5),
            ExeuctionMode::Benchmark => (Box::new(-5_000_000..5_000_000), 10_000_000),
        }
    }
}