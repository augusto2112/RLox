use crate::interpreter::Interpreter;
use crate::value::LoxValue::{Bool, Nil, Number};
use std::fmt;
use std::fmt::Debug;

#[derive(Debug, PartialEq, Clone)]
pub enum LoxValue {
    Number(f64),
    String(String),
    Bool(bool),
    Nil,
    Callable(Callable),
}

impl fmt::Display for LoxValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Number(number) => write!(f, "{}", number),
            LoxValue::String(string) => write!(f, "{}", string),
            Bool(boolean) => write!(f, "{}", boolean),
            Nil => write!(f, "nil"),
            LoxValue::Callable(callable) => std::fmt::Display::fmt(&callable, f),
        }
    }
}

impl LoxValue {
    pub fn is_truthy(&self) -> bool {
        match &self {
            Nil => false,
            Number(_) | LoxValue::String(_) | LoxValue::Callable(_) => true,
            Bool(boolean) => *boolean,
        }
    }
}
#[derive(Debug, PartialEq, Clone)]
pub enum Callable {
    Function { arity: u16 },
}

impl Callable {
    fn call(&self, interpreter: &Interpreter, arguments: &[LoxValue]) {}
}

impl fmt::Display for Callable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Callable::Function { arity } => write!(f, "Function with arity: {}", arity),
        }
    }
}
