use crate::interpreter::Interpreter;
use crate::statement::FuncStmt;
use crate::token::TokenType;
use crate::value::LoxValue::{Bool, Nil, Number};

use crate::environment::Environment;
use std::cell::RefCell;
use std::fmt;
use std::fmt::{Debug, Formatter};
use std::rc::Rc;
use thiserror::Error;

#[derive(Debug, Clone)]
pub enum LoxValue {
    Number(f64),
    String(String),
    Bool(bool),
    Nil,
    Callable(Callable),
}

impl PartialEq for LoxValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Number(lhs), Number(rhs)) => lhs == rhs,
            (LoxValue::String(lhs), LoxValue::String(rhs)) => lhs == rhs,
            (Bool(lhs), Bool(rhs)) => lhs == rhs,
            (Nil, Nil) => true,
            _ => false,
        }
    }
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
            Bool(boolean) => *boolean,
            _ => true,
        }
    }
}

#[derive(Error, Debug, Clone)]
pub enum LoxError {
    #[error("{0}")]
    Standard(String),
    // hack: we use return as an error so we can
    // unwind from the return keyword until we hit the calling function
    #[error("Return error")]
    Return(Return),
}

#[derive(Debug, Clone)]
pub struct Return {
    pub value: Option<LoxValue>,
}

#[derive(Clone)]
pub enum Callable {
    Function {
        arity: usize,
        func_stmt: FuncStmt,
        environment: Rc<RefCell<Environment>>,
    },
    Native {
        arity: usize,
        func: fn(&mut Interpreter, &[LoxValue]) -> Result<LoxValue, LoxError>,
    },
}

impl Debug for Callable {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        std::fmt::Display::fmt(&self, f)
    }
}

impl fmt::Display for Callable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Callable::Function { arity, .. } => write!(f, "Function : {}", arity),
            Callable::Native { .. } => write!(f, "Native function"),
        }
    }
}

impl Callable {
    pub fn arity(&self) -> usize {
        match &self {
            Callable::Function { arity, .. } | Callable::Native { arity, .. } => *arity,
        }
    }
    pub fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: &[LoxValue],
    ) -> Result<LoxValue, LoxError> {
        match &self {
            Callable::Function {
                func_stmt,
                environment,
                ..
            } => {
                let new_environment = Environment::new_enclosed(Rc::clone(&environment));

                for (param, arg) in func_stmt.params.iter().zip(arguments.iter()) {
                    if let TokenType::Identifier(lexeme) = &param.token_type {
                        new_environment.borrow_mut().define(lexeme, arg);
                    }
                }
                let value = interpreter.execute_block(&func_stmt.body, new_environment);
                if let Err(LoxError::Return(Return { value: Some(value) })) = value {
                    Ok(value)
                } else if let Err(LoxError::Return(Return { value: None })) = value {
                    Ok(LoxValue::Nil)
                } else {
                    value.map(|_| LoxValue::Nil)
                }
            }
            Callable::Native { func, .. } => func(interpreter, arguments),
        }
    }
}
