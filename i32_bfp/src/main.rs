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

    #[test]
    fn function_call_is_compiled_correctly_1() {
        let mut compiled_executor = CompiledExecutor::new();
        let mut interpreted_executor = InterpretedExecutor::new();
        handle_fn_def("f(x) := x + 1", &mut compiled_executor, &mut interpreted_executor);
        check_query_equiv("f(1) = 2", vec![0], &mut compiled_executor, &mut interpreted_executor);
    }

    #[test]
    fn function_call_is_compiled_correctly_2() {
        let mut compiled_executor = CompiledExecutor::new();
        let mut interpreted_executor = InterpretedExecutor::new();
        handle_fn_def("f(x) := x + 1", &mut compiled_executor, &mut interpreted_executor);
        handle_fn_def("g(x) := f(x) / 2", &mut compiled_executor, &mut interpreted_executor);
        check_query_equiv("f(x) > g(x)", vec![i32::MIN, -1, 0, 1, i32::MAX], &mut compiled_executor, &mut interpreted_executor);
    }

    #[test]
    fn function_call_encountered_bug_1() {
        let mut compiled_executor = CompiledExecutor::new();
        let mut interpreted_executor = InterpretedExecutor::new();
        handle_fn_def("f(x) := x + 1", &mut compiled_executor, &mut interpreted_executor);
        check_query_equiv("f(x) > x", vec![1189796073], &mut compiled_executor, &mut interpreted_executor);
    }

    fn check_equiv(expr: &str, test_for: Vec<i32>) {
        let mut compiled_executor = CompiledExecutor::new();
        let mut interpreted_executor = InterpretedExecutor::new();
        check_query_equiv(expr, test_for, &mut compiled_executor, &mut interpreted_executor)
    }

    fn handle_fn_def(expr: &str, compiled_executor: &mut CompiledExecutor, interpreted_executor: &mut InterpretedExecutor) {
        let parsed = parse(expr).unwrap();

        let defintion = match parsed {
            crate::ast::Action::FunctionDef(defintion) => defintion,
            _ => panic!("Expected query")
        };
        compiled_executor.handle_function_def(defintion.clone()).unwrap();
        interpreted_executor.handle_function_def(defintion).unwrap();
    }

    fn check_query_equiv(expr: &str, test_for: Vec<i32>, compiled_executor: &mut CompiledExecutor, interpreted_executor: &mut InterpretedExecutor) {
        let parsed = parse(expr).unwrap();

        let expr = match parsed {
            crate::ast::Action::Query(expr) => expr,
            _ => panic!("Expected query")
        };

        let compiled = compiled_executor.get_query_runable(expr.clone()).unwrap();
        let interpreted = interpreted_executor.get_query_runable(expr).unwrap();
        for val in test_for {
            let res1 = compiled(val);
            let res2 = interpreted(val);
            assert_eq!(res1, res2, "The values were not equal for input {}. Compiled: {}, Interpreted: {}.", val, res1, res2)
        }
    }
}