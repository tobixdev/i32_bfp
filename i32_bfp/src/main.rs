mod bfp_parser;
mod ast;

#[macro_use]
extern crate pest_derive;
use std::io::{self, BufRead, Write};
use bfp_parser::parse;
use io::Stdin;

fn main() {
    let stdin = io::stdin();
    loop {
        print!("> ");
        std::io::stdout().flush().expect("flush error.");
        let input = read_line(&stdin);
        match input.as_deref() {
            Some("quit") => { return; },
            Some(line) => {handle_line(line);}
            None => {}
        }
    }
}

fn handle_line(line: &str) -> () {
    match parse(line) {
        Ok(_) => {
            
        }
        Err(error) => {
            println!("Error while parsing: {}", error)
        }
    }
}

fn read_line(stdin: &Stdin) -> Option<String> {
    let mut iterator = stdin.lock().lines();
    iterator.next().map(|opt| opt.unwrap())
}
