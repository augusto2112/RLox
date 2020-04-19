use crate::token::Token;
use crate::token::TokenType;
use crate::value::{LoxError, LoxValue};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct Environment {
    enclosed: Option<Rc<RefCell<Environment>>>,
    values: HashMap<String, LoxValue>,
}

impl Environment {
    pub fn new() -> Rc<RefCell<Environment>> {
        Rc::from(RefCell::from(Environment {
            enclosed: Option::None,
            values: HashMap::new(),
        }))
    }

    pub fn new_enclosed(enclosing: Rc<RefCell<Environment>>) -> Rc<RefCell<Environment>> {
        Rc::from(RefCell::from(Environment {
            enclosed: Option::from(enclosing),
            values: HashMap::new(),
        }))
    }

    pub fn define(&mut self, name: &str, value: &LoxValue) {
        self.values.insert(name.to_string(), value.clone());
    }

    pub fn assign(&mut self, name: &str, value: &LoxValue) -> Result<(), LoxError> {
        if self.values.contains_key(name) {
            self.values.insert(name.to_string(), value.clone());
            return Ok(());
        }

        if let Some(ref mut enclosed) = self.enclosed {
            let result = enclosed.borrow_mut().assign(name, value);
            if result.is_ok() {
                return result;
            }
        }

        Err(LoxError::Standard(format!(
            "Undefined variable '{}'.",
            name
        )))
    }

    pub fn get(&self, token: &Token) -> Result<LoxValue, LoxError> {
        if let TokenType::Identifier(identifier) = &token.token_type {
            if let Some(value) = self.values.get(identifier) {
                Ok(value.clone())
            } else if let Some(enclosed) = &self.enclosed {
                enclosed.borrow_mut().get(token)
            } else {
                Err(LoxError::Standard(format!(
                    "Undefined variable: {}",
                    identifier
                )))
            }
        } else {
            panic!("Compiler bug: unexpected token: {:?}", token);
        }
    }
}
