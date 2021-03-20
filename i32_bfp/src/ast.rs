pub enum Action {
    FunctionDef(FunctionDef)
}

pub struct FunctionDef {
    pub name: String,
    pub body: Box<Expr>
}

pub enum Expr {
    Number(i32)
}