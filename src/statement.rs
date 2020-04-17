use crate::expression::Expr;
use crate::token::Token;

#[derive(Debug)]
pub enum Stmt {
    Expr(Expr),
    Print(Expr),
    If(Expr, Box<Stmt>, Option<Box<Stmt>>),
    Var(Token, Option<Expr>),
    Block(Vec<Stmt>),
    While(Expr, Box<Stmt>),
}
