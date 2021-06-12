use std::{collections::HashMap, num::Wrapping};

use crate::{ast::{self, Expr}, runtime::Executor};

pub struct InterpretedExecutor {
    asts: HashMap<String, ast::FunctionDef>
}

impl InterpretedExecutor {
    pub fn new() -> InterpretedExecutor {
        InterpretedExecutor {
            asts: HashMap::new()
        }
    }
}

impl Executor for InterpretedExecutor {
    fn handle_function_def(&mut self, func_def: ast::FunctionDef) -> Result<(), String> {
        self.asts.insert(func_def.name.to_string(), func_def);
        Ok(())
    }
    
    fn get_query_runable<'a>(&'a mut self, query: ast::Expr) -> Result<Box<dyn 'a + Fn(i32) -> i32>, String> {
        Ok(Box::new(move |x| {
            let ctx = InterpretationContext::new(&self);
            ctx.eval(&query, x) 
        }))
    }

    fn delete(&mut self, name: &str) {
        self.asts.remove(name);
    }
}

struct InterpretationContext<'a> {
    executor: &'a InterpretedExecutor,
    vars: HashMap<String, i32>
}

impl<'a> InterpretationContext<'a> {
    fn new(executor: &InterpretedExecutor) -> InterpretationContext {
        InterpretationContext {
            executor: executor,
            vars: HashMap::new()
        }
    }

    fn run(&self, name: &str, arg: i32) -> i32 {
        let ast = self.executor.asts.get(name).unwrap();
        self.eval(&ast.body, arg)
    }

    fn eval(&self, expr: &Expr, arg: i32) -> i32 {
        let mut inner = InterpretationContext::new(&self.executor);
        let callee_vars = expr.used_variables();
        if callee_vars.len() > 0 {
            inner.vars.insert(callee_vars[0].to_string(), arg);
        }
        expr.eval(&inner)
    }
}

trait Interpretable {
    fn eval(&self, ctx: &InterpretationContext) -> i32;
}

impl Interpretable for Expr {
    fn eval(&self, ctx: &InterpretationContext) -> i32 {
        match self {
            Expr::Number(x) => *x,
            Expr::Var(v) => *ctx.vars.get(v).unwrap(),
            Expr::FunctionCall(name, arg_expr) => {
                let arg = match arg_expr {
                    Some(exp) => exp.eval(ctx),
                    None => 0
                };
                ctx.run(name, arg)
            },
            Expr::Add(a, b) => (Wrapping(a.eval(ctx)) + Wrapping(b.eval(ctx))).0,
            Expr::Sub(a, b) => (Wrapping(a.eval(ctx)) - Wrapping(b.eval(ctx))).0,
            Expr::Mul(a, b) => (Wrapping(a.eval(ctx)) * Wrapping(b.eval(ctx))).0,
            Expr::Div(a, b) => (Wrapping(a.eval(ctx)) / Wrapping(b.eval(ctx))).0,
            Expr::Rem(a, b) => (Wrapping(a.eval(ctx)) % Wrapping(b.eval(ctx))).0,
            Expr::Eq(a, b) => if a.eval(ctx) == b.eval(ctx) { 1 } else { 0 },
            Expr::Neq(a, b) => if a.eval(ctx) != b.eval(ctx) { 1 } else { 0 }
            Expr::Gt(a, b) => if a.eval(ctx) > b.eval(ctx) { 1 } else { 0 },
            Expr::Lt(a, b) => if a.eval(ctx) < b.eval(ctx) { 1 } else { 0 },
            Expr::Gte(a, b) => if a.eval(ctx) >= b.eval(ctx) { 1 } else { 0 },
            Expr::Lte(a, b) => if a.eval(ctx) <= b.eval(ctx) { 1 } else { 0 },
        }
    }
}