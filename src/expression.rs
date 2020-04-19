use crate::token::Token;

#[derive(Debug, Clone)]
pub enum Expr {
    Number(f64),
    String(String),
    True,
    False,
    Nil,
    Grouping(Box<Expr>),
    Unary(Token, Box<Expr>),
    Binary(Box<Expr>, Token, Box<Expr>),
    Variable(Token),
    Assignment(Token, Box<Expr>),
    Logical(Box<Expr>, Token, Box<Expr>),
    Call(Box<Expr>, Token, Vec<Expr>),
}
