use itertools::Itertools;

#[derive(Debug)]
pub enum Action {
    FunctionDef(FunctionDef),
    Query(Expr),
    Command(Command)
}

#[derive(Debug)]
pub enum Command {
    ShowCode(String),
    ListFunctions(),
    DeleteFunction(String)
}

#[derive(Debug)]
pub struct FunctionDef {
    pub name: String,
    pub parameter: Option<String>,
    pub body: Expr
}

#[derive(Debug)]
pub enum Expr {
    Number(i32),
    Var(String),
    FunctionCall(String, Option<Box<Expr>>),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
    Eq(Box<Expr>, Box<Expr>),
    Neq(Box<Expr>, Box<Expr>),
}

impl Expr {
    pub fn used_variables(&self) -> Vec<String> {
        let mut vars = Vec::new();
        self.add_used_variables(&mut vars);
        vars.into_iter().unique().collect_vec()
    }

    fn add_used_variables(&self, mut vars: &mut Vec<String>) {
        match self {
            Expr::Number(_) => {}
            Expr::Var(v) => { vars.push(v.clone()) }
            Expr::Add(lhs, rhs) | 
            Expr::Sub(lhs, rhs) | 
            Expr::Mul(lhs, rhs) | 
            Expr::Div(lhs, rhs) |
            Expr::Eq(lhs, rhs) |
            Expr::Neq(lhs, rhs) => { 
                lhs.add_used_variables(&mut vars);
                rhs.add_used_variables(&mut vars);
            },
            Expr::FunctionCall(_, expr) => {
                if let Some(expr) = expr {
                    expr.add_used_variables(&mut vars)
                }
            }
        }
    }
}