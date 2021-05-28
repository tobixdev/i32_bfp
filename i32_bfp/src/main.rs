mod bfp_parser;
mod ast;
mod code_repository;
mod compiler;

#[macro_use]
extern crate pest_derive;
#[macro_use]
extern crate dynasm;
use std::io::{self, BufRead, Write};
use bfp_parser::parse;
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

fn handle_line(mut code_repository: &mut CodeRepository, line: &str) -> () {
    match parse(line) {
        Ok(ast) => {
            handle_ast(&mut code_repository, ast);
        }
        Err(error) => {
            println!("Error while parsing: {}", error)
        }
    }
}

fn handle_ast(code_repository: &mut CodeRepository, ast: ast::Action) {
    match ast {
        ast::Action::FunctionDef(func_def) => {
            code_repository.add_placeholder(&func_def)
        }
        ast::Action::Query(query) => {
            let used_vars = query.used_variables();
            let mut ctx = CompilationContext::new();
            used_vars.iter().for_each(|v| ctx.assign_register_to_variable(v.to_string()));
            let runable = ctx.compile(&query);
            println!("The following free variables were found: {:?}", used_vars);
            println!("Result: {}", runable.call(1));
        }
        ast::Action::Command(ast::Command::ShowCode(name)) => {
            code_repository.print_code(&name);
        }
    }
}

fn read_line(stdin: &Stdin) -> Option<String> {
    let mut iterator = stdin.lock().lines();
    iterator.next().map(|opt| opt.unwrap())
}
