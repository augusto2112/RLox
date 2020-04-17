use crate::token::Token;
use crate::token::TokenType;
use crate::value::LoxValue;
use std::collections::HashMap;
use std::mem::discriminant;

#[derive(Debug)]
pub struct Environment<'a> {
    enclosed: Option<&'a mut Environment<'a>>,
    values: HashMap<String, LoxValue>,
}

impl<'a> Environment<'a> {
    pub fn new() -> Environment<'a> {
        Environment {
            enclosed: Option::None,
            values: HashMap::new(),
        }
    }

    pub fn new_enclosed(enclosing: &'a mut Environment<'a>) -> Environment<'a> {
        Environment {
            enclosed: Option::from(enclosing),
            values: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: &str, value: &LoxValue) {
        self.values.insert(name.to_string(), value.clone());
    }

    pub fn assign(&mut self, name: &str, value: &LoxValue) -> Result<(), String> {
        if self.values.contains_key(name) {
            self.values.insert(name.to_string(), value.clone());
            return Ok(());
        }

        if let Some(ref mut enclosed) = self.enclosed {
            enclosed.assign(name, value);
        }

        Err(format!("Undefined variable '{}'.", name))
    }

    pub fn get(&self, token: &Token) -> Result<LoxValue, String> {
        panic!();

        // if let Some(enclosed) = &self.enclosed {
        //     let result = enclosed.get(token);
        //     if result.is_ok() {
        //         return result;
        //     }
        // }
        //
        // if let TokenType::Identifier(identifier) = &token.token_type {
        //     self.values
        //         .get(identifier)
        //         .ok_or({ format!("Undefined variable: {}", identifier) })
        //         .map(std::clone::Clone::clone)
        // } else if let Some(enclosed) = &self.enclosed {
        //     enclosed.get(token)
        // } else {
        //     panic!("Compiler bug: unexpected token: {:?}", token);
        // }
    }
}
