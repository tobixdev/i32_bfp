mod parser;
mod ast;
mod code_repository;
mod compiler;

#[macro_use]
extern crate pest_derive;
extern crate dynasm;
use std::io::{self, BufRead, Write};
use parser::parse;
use code_repository::CodeRepository;
use compiler::CompilationContext;
use io::Stdin;

fn main() {
    let stdin = io::stdin();
    let mut code_repository = CodeRepository::new();
    loop {
        print!("> ");
        std::io::stdout().flush().expect("flush error.");
        let input = read_line(&stdin);
        match input.as_deref() {
            Some("quit") => { return; },
            Some(line) => {handle_line(&mut code_repository, line);}
            None => {}
        }
    }
}

fn handle_line(mut code_repository: &mut CodeRepository, line: &str) {
    let result = parse(line)
        .and_then(|ast| handle_ast(&mut code_repository, ast));
    if let Err(error) = result {
        println!("ERROR>\n{}", error);
    }
}

fn handle_ast(code_repository: &mut CodeRepository, ast: ast::Action) -> Result<(), String> {
    match ast {
        ast::Action::FunctionDef(func_def) => {
            code_repository.add_placeholder(&func_def)?
        }
        ast::Action::Query(query) => {
            execute_query(query)?;
        }
        ast::Action::Command(ast::Command::ShowCode(name)) => {
            code_repository.print_code(&name);
        }
    }
    Ok(())
}

fn execute_query(query: ast::Expr) -> Result<(), String> {
    let used_vars = query.used_variables();
    let mut ctx = CompilationContext::new();

    for used_var in &used_vars {
        ctx.assign_register_to_variable(used_var.to_string())?;
    }

    let runable = ctx.compile(&query)?;
    println!("The following free variables were found: {:?}", used_vars);
    let first_var_range = if used_vars.len() > 0 { i32::MIN..=i32::MAX } else { 0..=0 };
    let mut to_check = if used_vars.len() > 0 { (i32::MAX as usize) + 1 + -(i32::MIN as i64) as usize } else { 1 };
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

fn read_line(stdin: &Stdin) -> Option<String> {
    let mut iterator = stdin.lock().lines();
    iterator.next().map(|opt| opt.unwrap())
}
