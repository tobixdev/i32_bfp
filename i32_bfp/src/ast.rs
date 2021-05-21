pub enum Action {
    FunctionDef(FunctionDef)
}

pub struct FunctionDef {
    pub name: String,
    pub parameter: Option<String>,
    pub body: Box<Expr>
}

pub enum Expr {
    Number(i32),
    Var(String),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>)
}