mod parser;
mod ast;
mod code_repository;
mod compiler;
mod runtime;
mod compiled_executor;

#[macro_use]
extern crate pest_derive;
extern crate dynasm;
use std::io::{self, BufRead, Stdin, Write};

fn main() {
    let stdin = io::stdin();
    let mut runtime = runtime::Runtime::new_compiled();
    loop {
        print!("> ");
        std::io::stdout().flush().expect("flush error.");
        let input = read_line(&stdin);
        match input.as_deref() {
            Some("quit") => { return; },
            Some(line) => {runtime.handle_line(line);}
            None => {}
        }
    }
}
    
fn read_line(stdin: &Stdin) -> Option<String> {
    let mut iterator = stdin.lock().lines();
    iterator.next().map(|opt| opt.unwrap())
}