use std::io::{self, BufRead, Write};

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
    
}

fn read_line(stdin: &Stdin) -> Option<String> {
    let mut iterator = stdin.lock().lines();
    iterator.next().map(|opt| opt.unwrap())
}
