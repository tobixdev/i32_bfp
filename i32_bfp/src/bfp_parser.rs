extern crate pest;

use crate::ast;
use pest::{iterators::Pairs, Parser};

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct IdentParser;

pub fn parse(input: &str) -> Result<ast::Action, String> {
    let mut pairs = IdentParser::parse(Rule::action, input).map_err(|e| e.to_string())?;
    Ok(build_ast_root(&mut pairs))
}

fn build_ast_root(pairs: &mut Pairs<'_, Rule>) -> ast::Action {
    let rule = pairs.next().unwrap();
    return match rule.as_rule() {
        Rule::action => build_ast_action(&mut rule.into_inner()),
        _ => unreachable!("Rule cannot be matched in root"),
    };
}

fn build_ast_action(pairs: &mut Pairs<'_, Rule>) -> ast::Action {
    let rule = pairs.next().unwrap();
    return match rule.as_rule() {
        Rule::function_def => {
            ast::Action::FunctionDef(build_ast_function_def(&mut rule.into_inner()))
        }
        _ => unreachable!("Rule cannot be matched in action"),
    };
}

fn build_ast_function_def(pairs: &mut Pairs<'_, Rule>) -> ast::FunctionDef {
    let name_rule = pairs.next().unwrap();
    let expr_rule = pairs.next().unwrap();
    let expr = match expr_rule.as_rule() {
        Rule::expr => build_ast_expr(&mut expr_rule.into_inner()),
        _ => unreachable!("Rule cannot be matched in function def"),
    };

    ast::FunctionDef {
        name: name_rule.as_str().to_string(),
        body: Box::new(expr),
    }
}

fn build_ast_expr(pairs: &mut Pairs<'_, Rule>) -> ast::Expr {
    let rule = pairs.next().unwrap();
    match rule.as_rule() {
        Rule::number => ast::Expr::Number(rule.as_str().parse().unwrap()),
        _ => unreachable!("Rule cannot be matched in expr"),
    }
}
