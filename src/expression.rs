use crate::token::Token; 

#[derive(Debug)]
pub enum Expr {
    Number(f64),
    String_(String),
    True,
    False,
    Nil,
    Grouping(Box<Expr>),
    Unary(Token, Box<Expr>),
    Binary(Box<Expr>, Token, Box<Expr>),
}