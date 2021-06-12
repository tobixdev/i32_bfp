mod parser;
mod ast;
mod code_repository;
mod compiler;
mod runtime;
mod compiled_executor;
mod interpreted_executor;

#[macro_use]
extern crate pest_derive;
extern crate dynasm;
use std::io::{self, BufRead, Stdin, Write};

fn main() {
    let stdin = io::stdin();
    let mut runtime = runtime::Runtime::new();
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

#[cfg(test)]
mod tests {
    use crate::{compiled_executor::CompiledExecutor, interpreted_executor::InterpretedExecutor, parser::parse, runtime::Executor};

    #[test]
    fn num_is_compiled_correctly() {
        check_equiv("10", vec![0])
    }

    #[test]
    fn var_is_compiled_correctly() {
        check_equiv("x", vec![i32::MIN, -1, 0, 1, i32::MAX])
    }

    #[test]
    fn add_is_compiled_correctly() {
        check_equiv("x + 1 + x", vec![i32::MIN, -1, 0, 1, i32::MAX])
    }

    #[test]
    fn sub_is_compiled_correctly() {
        check_equiv("x - x + x - 10", vec![i32::MIN, -1, 0, 1, i32::MAX])
    }

    #[test]
    fn mul_is_compiled_correctly() {
        check_equiv("x * 10 * 4", vec![i32::MIN, -1, 0, 1, i32::MAX])
    }

    #[test]
    fn div_is_compiled_correctly() {
        check_equiv("x / 10", vec![i32::MIN, -1, 0, 1, i32::MAX])
    }

    #[test]
    fn rem_is_compiled_correctly() {
        check_equiv("x % 10", vec![i32::MIN, -1, 0, 1, i32::MAX])
    }

    #[test]
    fn eq_is_compiled_correctly() {
        check_equiv("x = 1", vec![i32::MIN, -1, 0, 1, i32::MAX])
    }

    #[test]
    fn neq_is_compiled_correctly() {
        check_equiv("x <> 1", vec![i32::MIN, -1, 0, 1, i32::MAX])
    }

    #[test]
    fn gt_is_compiled_correctly() {
        check_equiv("x > 1", vec![i32::MIN, -1, 0, 1, i32::MAX])
    }

    #[test]
    fn lt_is_compiled_correctly() {
        check_equiv("x < 1", vec![i32::MIN, -1, 0, 1, i32::MAX])
    }

    #[test]
    fn gte_is_compiled_correctly() {
        check_equiv("x >= 1", vec![i32::MIN, -1, 0, 1, i32::MAX])
    }

    #[test]
    fn lte_is_compiled_correctly() {
        check_equiv("x <= 1", vec![i32::MIN, -1, 0, 1, i32::MAX])
    }

    fn check_equiv(expr: &str, test_for: Vec<i32>) {
        let parsed = parse(expr).unwrap();

        let expr = match parsed {
            crate::ast::Action::Query(expr) => expr,
            _ => panic!("Expected query")
        };

        let mut compiled_executor = CompiledExecutor::new();
        let mut interpreted_executor = InterpretedExecutor::new();
        let compiled = compiled_executor.get_query_runable(expr.clone()).unwrap();
        let interpreted = interpreted_executor.get_query_runable(expr).unwrap();
        for val in test_for {
            let res1 = compiled(val);
            let res2 = interpreted(val);
            assert_eq!(res1, res2, "The values were not equal for input {}. Compiled: {}, Interpreted: {}.", val, res1, res2)
        }
    }
}