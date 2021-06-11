extern crate pest;

use crate::ast;
use pest::{iterators::Pairs, Parser};

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct IdentParser;

pub fn parse(input: &str) -> Result<ast::Action, String> {
    let mut pairs = IdentParser::parse(Rule::action, input).map_err(|e| e.to_string())?;
    Ok(build_ast_root(&mut pairs)?)
}

fn build_ast_root(pairs: &mut Pairs<'_, Rule>) -> Result<ast::Action, String> {
    let rule = pairs.next().unwrap();
    return match rule.as_rule() {
        Rule::action => build_ast_action(&mut rule.into_inner()),
        _ => unreachable!("Rule cannot be matched in root"),
    };
}

fn build_ast_action(pairs: &mut Pairs<'_, Rule>) -> Result<ast::Action, String> {
    let rule = pairs.next().unwrap();
    Ok(match rule.as_rule() {
        Rule::function_def => {
            ast::Action::FunctionDef(build_ast_function_def(&mut rule.into_inner())?)
        }
        Rule::query => build_ast_query(&mut rule.into_inner())?,
        Rule::command => ast::Action::Command(build_ast_command(&mut rule.into_inner())?),
        _ => unreachable!("Rule cannot be matched in action"),
    })
}

fn build_ast_function_def(pairs: &mut Pairs<'_, Rule>) -> Result<ast::FunctionDef, String> {
    let name_rule = pairs.next().unwrap();
    let second_pair = pairs.next().unwrap();
    let rule_to_match = second_pair.as_rule();
    let result = match rule_to_match {
        Rule::expr => (build_ast_expr(&mut second_pair.into_inner())?, None),
        Rule::ID => (
            build_ast_expr(&mut pairs.next().unwrap().into_inner())?,
            Some(second_pair.as_str().to_string()),
        ),
        _ => unreachable!("Rule cannot be matched in function def"),
    };

    Ok(ast::FunctionDef {
        name: name_rule.as_str().to_string(),
        parameter: result.1,
        body: result.0,
    })
}

fn build_ast_query(pairs: &mut Pairs<'_, Rule>) -> Result<ast::Action, String> {
    let rule = pairs.next().unwrap();
    Ok(ast::Action::Query(build_ast_expr(&mut rule.into_inner())?))
}

fn build_ast_expr(pairs: &mut Pairs<'_, Rule>) -> Result<ast::Expr, String> {
    let rule = pairs.next().unwrap();
    Ok(match rule.as_rule() {
        Rule::relation => build_ast_relation(&mut rule.into_inner())?,
        _ => unreachable!("Rule cannot be matched in expr"),
    })
}

fn build_ast_relation(pairs: &mut Pairs<'_, Rule>) -> Result<ast::Expr, String> {
    let lhs = pairs.next().unwrap();
    let op = pairs.next();

    if op.is_none() {
        return build_ast_addsub(&mut lhs.into_inner());
    }
    let rhs = pairs.next().unwrap();
    Ok(match op.unwrap().as_str() {
        "=" => ast::Expr::Eq(
            Box::new(build_ast_addsub(&mut lhs.into_inner())?),
            Box::new(build_ast_relation(&mut rhs.into_inner())?),
        ),
        "<>" => ast::Expr::Neq(
            Box::new(build_ast_addsub(&mut lhs.into_inner())?),
            Box::new(build_ast_relation(&mut rhs.into_inner())?),
        ),
        _ => unreachable!("Operator cannot be matched in relation"),
    })
}

fn build_ast_addsub(pairs: &mut Pairs<'_, Rule>) -> Result<ast::Expr, String> {
    let lhs = pairs.next().unwrap();
    let op = pairs.next();

    if op.is_none() {
        return build_ast_muldiv(&mut lhs.into_inner());
    }
    let rhs = pairs.next().unwrap();
    Ok(match op.unwrap().as_str() {
        "+" => ast::Expr::Add(
            Box::new(build_ast_muldiv(&mut lhs.into_inner())?),
            Box::new(build_ast_addsub(&mut rhs.into_inner())?),
        ),
        "-" => ast::Expr::Sub(
            Box::new(build_ast_muldiv(&mut lhs.into_inner())?),
            Box::new(build_ast_addsub(&mut rhs.into_inner())?),
        ),
        _ => unreachable!("Operator cannot be matched in addsub"),
    })
}

fn build_ast_muldiv(pairs: &mut Pairs<'_, Rule>) -> Result<ast::Expr, String> {
    let lhs = pairs.next().unwrap();
    let op = pairs.next();

    if op.is_none() {
        return build_ast_atom(&mut lhs.into_inner());
    }

    let rhs = pairs.next().unwrap();
    Ok(match op.unwrap().as_str() {
        "*" => ast::Expr::Mul(
            Box::new(build_ast_atom(&mut lhs.into_inner())?),
            Box::new(build_ast_muldiv(&mut rhs.into_inner())?),
        ),
        "/" => ast::Expr::Div(
            Box::new(build_ast_atom(&mut lhs.into_inner())?),
            Box::new(build_ast_muldiv(&mut rhs.into_inner())?),
        ),
        _ => unreachable!("Operator cannot be matched in muldiv"),
    })
}

fn build_ast_atom(pairs: &mut Pairs<'_, Rule>) -> Result<ast::Expr, String> {
    let rule = pairs.next().unwrap();
    Ok(match rule.as_rule() {
        Rule::NUMBER => ast::Expr::Number(rule.as_str().parse().or_else(|_| Err("Integer too big.".to_string()))?),
        Rule::ID => ast::Expr::Var(rule.as_str().to_string()),
        Rule::expr => build_ast_expr(&mut rule.into_inner())?,
        Rule::function_call => build_ast_function_call(&mut rule.into_inner())?,
        _ => unreachable!("Rule cannot be matched in atom"),
    })
}

fn build_ast_function_call(pairs: &mut Pairs<'_, Rule>) -> Result<ast::Expr, String> {
    let rule = pairs.next().unwrap();
    let arg = match pairs.next() {
        Some(next_rule) => Some(Box::new(build_ast_expr(&mut next_rule.into_inner())?)),
        None => None,
    };
    Ok (ast::Expr::FunctionCall(rule.as_str().to_string(), arg))
}

fn build_ast_command(pairs: &mut Pairs<'_, Rule>) -> Result<ast::Command, String> {
    let rule = pairs.next().unwrap();
    Ok(match rule.as_rule() {
        Rule::show_code_command => ast::Command::ShowCode(rule.into_inner().next().unwrap().as_str().to_string()),
        Rule::list_fn_command => ast::Command::ListFunctions(),
        Rule::delete_fn_command => ast::Command::DeleteFunction(rule.into_inner().next().unwrap().as_str().to_string()),
        Rule::mode_command => ast::Command::SwitchMode(rule.into_inner().next().unwrap().as_str().to_string()),
        Rule::executor_command => ast::Command::SwitchExecutor(rule.into_inner().next().unwrap().as_str().to_string()),
        Rule::test_command => ast::Command::Test(build_ast_expr(&mut rule.into_inner().next().unwrap().into_inner())?),
        _ => unreachable!("Rule cannot be matched in command"),
    })
}
