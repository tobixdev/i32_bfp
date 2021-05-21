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
    }
}

fn read_line(stdin: &Stdin) -> Option<String> {
    let mut iterator = stdin.lock().lines();
    iterator.next().map(|opt| opt.unwrap())
}
