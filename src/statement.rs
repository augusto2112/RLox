use crate::expression::Expr;
use crate::token::Token;

#[derive(Debug, Clone)]
pub enum Stmt {
    Block(Vec<Stmt>),
    Expr(Expr),
    Function(FuncStmt),
    If(Expr, Box<Stmt>, Option<Box<Stmt>>),
    Print(Expr),
    Var(Token, Option<Expr>),
    While(Expr, Box<Stmt>),
    Ret(Option<Expr>),
}

#[derive(Debug, Clone)]
pub struct FuncStmt {
    pub name: Token,
    pub params: Vec<Token>,
    pub body: Vec<Stmt>,
}
