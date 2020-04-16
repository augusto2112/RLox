use crate::expression::Expr; 
use crate::token::Token;

#[derive(Debug)]
pub enum Stmt {
    Expr(Expr),
    Print(Expr),
    Var(Token, Option<Expr>),
    Block(Vec<Stmt>)
}